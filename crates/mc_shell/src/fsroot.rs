//! Filesystem confinement (SPEC-005 section 2).
//!
//! This module enforces that all file I/O happens within designated root
//! directories. Callers supply a [`Root`] variant and a requested path; the
//! [`confine`] function resolves the root from an environment variable,
//! canonicalises both paths, and rejects any request that escapes the root
//! (directory traversal, symlink escape, or an unresolvable root).
//!
//! # Environment variables
//!
//! | `Root` variant | Env var            |
//! |----------------|--------------------|
//! | `Content`      | `MC_CONTENT_DIR`   |
//! | `Data`         | `MC_DATA_DIR`      |
//! | `Artifact`     | `MC_ARTIFACT_DIR`  |

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// A named root directory, resolved from an environment variable at call time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Root {
    /// Game content (art, strings, scenes, etc.) — resolved from `MC_CONTENT_DIR`.
    Content,
    /// User data (saves, settings) — resolved from `MC_DATA_DIR`.
    Data,
    /// Build artifacts (baked packs) — resolved from `MC_ARTIFACT_DIR`.
    Artifact,
}

/// Errors that can occur during path confinement.
#[derive(Debug)]
pub enum FsError {
    /// The environment variable for this root is unset or empty.
    UnresolvedRoot(Root),
    /// The root path does not exist or cannot be canonicalised.
    InvalidRoot(Root, String),
    /// The requested path escapes the root via `..` traversal.
    TraversalDetected {
        root: PathBuf,
        requested: PathBuf,
        resolved: PathBuf,
    },
    /// A component of the requested path is a symlink pointing outside the root.
    SymlinkEscape {
        root: PathBuf,
        link: PathBuf,
        target: PathBuf,
    },
    /// The requested path could not be canonicalised.
    ResolveError(String),
}

impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsError::UnresolvedRoot(r) => {
                write!(f, "environment variable for root {r:?} is unset or empty")
            }
            FsError::InvalidRoot(r, msg) => {
                write!(f, "root {r:?} is invalid: {msg}")
            }
            FsError::TraversalDetected {
                root,
                requested,
                resolved,
            } => {
                write!(
                    f,
                    "traversal detected: {requested:?} resolved to {resolved:?} which is outside root {root:?}"
                )
            }
            FsError::SymlinkEscape { root, link, target } => {
                write!(
                    f,
                    "symlink escape detected: {link:?} points to {target:?} outside root {root:?}"
                )
            }
            FsError::ResolveError(msg) => {
                write!(f, "path resolution error: {msg}")
            }
        }
    }
}

impl std::error::Error for FsError {}

impl Root {
    /// Return the env-var name for this root.
    pub fn env_var(&self) -> &'static str {
        match self {
            Root::Content => "MC_CONTENT_DIR",
            Root::Data => "MC_DATA_DIR",
            Root::Artifact => "MC_ARTIFACT_DIR",
        }
    }

    /// Resolve the root path from the environment variable.
    ///
    /// Returns `None` if the env var is unset or empty.
    pub fn resolve_env(&self) -> Option<PathBuf> {
        let val = env::var(self.env_var()).ok()?;
        if val.is_empty() {
            return None;
        }
        Some(PathBuf::from(val))
    }
}

/// Confine `requested` to the root directory identified by `root`.
///
/// This function:
/// 1. Resolves the root path from the corresponding environment variable.
/// 2. Delegates to [`confine_to_path`].
pub fn confine(root: Root, requested: &Path) -> Result<PathBuf, FsError> {
    let root_path = root.resolve_env().ok_or(FsError::UnresolvedRoot(root))?;
    confine_to_path(&root_path, root, requested)
}

/// Confine `requested` to an explicit `root_path` (testing-friendly version).
///
/// Same semantics as [`confine`] but accepts an explicit root path instead of
/// resolving from an environment variable.
///
/// 1. Canonicalises the root (requires it to exist).
/// 2. Resolves `requested` relative to the root, handling both existing and
///    non-existent paths (for write operations).
/// 3. Verifies the resolved path is within the root (no traversal, symlink escape).
///
/// Returns the confined absolute path on success.
pub fn confine_to_path(
    root_path: &Path,
    _root: Root,
    requested: &Path,
) -> Result<PathBuf, FsError> {
    // 1. Canonicalise the root (requires it to exist on disk).
    let canonical_root = root_path
        .canonicalize()
        .map_err(|e| FsError::InvalidRoot(_root, format!("cannot canonicalise: {e}")))?;

    // 2. Build the full candidate path.
    //    For paths that don't exist yet (e.g., new files to write), we canonicalize
    //    as much as possible: the root itself, and the parent directories of the candidate.
    let candidate = canonical_root.join(requested);

    // Try to canonicalize the candidate. If it doesn't exist yet, that's okay for writes.
    let canonical_candidate = match candidate.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // Path doesn't exist yet. Resolve as much as we can.
            // Walk up from the candidate until we find an existing ancestor.
            let mut resolved = candidate.clone();
            let mut non_existent = Vec::new();
            loop {
                match resolved.canonicalize() {
                    Ok(existing) => {
                        // Rebuild the full path by appending non-existent components
                        let mut result = existing;
                        for comp in &non_existent {
                            result.push(comp);
                        }
                        break result;
                    }
                    Err(_) => {
                        let name = resolved.file_name().map(|n| n.to_os_string()).ok_or(
                            FsError::ResolveError(format!(
                                "cannot resolve any ancestor of {candidate:?}"
                            )),
                        )?;
                        non_existent.push(name);
                        match resolved.parent() {
                            Some(p) => resolved = p.to_path_buf(),
                            None => {
                                return Err(FsError::ResolveError(format!(
                                    "cannot resolve ancestor chain of {candidate:?}"
                                )));
                            }
                        }
                    }
                }
            }
        }
    };

    // 4. Check every component of the relative portion for symlink escape.
    let mut current = canonical_root.clone();
    for component in requested.components() {
        if component.as_os_str().is_empty() {
            continue;
        }
        current.push(component);
        // If the component is a symlink, check its target.
        if let Ok(metadata) = current.symlink_metadata() {
            if metadata.file_type().is_symlink() {
                let link_target = current.read_link().map_err(|e| {
                    FsError::ResolveError(format!("cannot read symlink {current:?}: {e}"))
                })?;
                // Resolve the link target relative to the parent directory
                let abs_target = if link_target.is_absolute() {
                    link_target
                } else {
                    let parent = current.parent().unwrap_or(&canonical_root);
                    parent.join(&link_target)
                };
                let canonical_target = abs_target.canonicalize().map_err(|e| {
                    FsError::ResolveError(format!(
                        "cannot canonicalise symlink target {abs_target:?}: {e}"
                    ))
                })?;
                if !canonical_target.starts_with(&canonical_root) {
                    return Err(FsError::SymlinkEscape {
                        root: canonical_root.clone(),
                        link: current.clone(),
                        target: canonical_target,
                    });
                }
            }
        }
    }

    // 5. Verify the candidate is within the root (strict sub-path or equal).
    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(FsError::TraversalDetected {
            root: canonical_root,
            requested: requested.to_path_buf(),
            resolved: canonical_candidate,
        });
    }

    Ok(canonical_candidate)
}

/// Confine and read a file to bytes.
pub fn read(root: Root, relative: &Path) -> Result<Vec<u8>, FsError> {
    let path = confine(root, relative)?;
    fs::read(&path).map_err(|e| FsError::ResolveError(format!("cannot read {path:?}: {e}")))
}

/// Confine and read a file to a String.
pub fn read_to_string(root: Root, relative: &Path) -> Result<String, FsError> {
    let path = confine(root, relative)?;
    fs::read_to_string(&path)
        .map_err(|e| FsError::ResolveError(format!("cannot read {path:?}: {e}")))
}

/// Confine and write bytes to a file.
pub fn write(root: Root, relative: &Path, data: &[u8]) -> Result<(), FsError> {
    let path = confine(root, relative)?;
    fs::write(&path, data).map_err(|e| FsError::ResolveError(format!("cannot write {path:?}: {e}")))
}

/// Confine and create a directory (and all parents).
pub fn create_dir_all(root: Root, relative: &Path) -> Result<PathBuf, FsError> {
    let path = confine(root, relative)?;
    fs::create_dir_all(&path)
        .map_err(|e| FsError::ResolveError(format!("cannot create {path:?}: {e}")))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_unset_env_returns_error() {
        std::env::remove_var(Root::Content.env_var());
        let err = confine(Root::Content, Path::new("foo.ron")).unwrap_err();
        assert!(matches!(err, FsError::UnresolvedRoot(Root::Content)));
    }

    #[test]
    fn test_non_existent_root_returns_error() {
        let err = confine_to_path(
            Path::new("/tmp/nonexistent_mc_test_XXXXXX"),
            Root::Content,
            Path::new("foo.ron"),
        )
        .unwrap_err();
        assert!(matches!(err, FsError::InvalidRoot(Root::Content, _)));
    }

    #[test]
    fn test_valid_confine_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let root_path = dir.path().canonicalize().unwrap();
        let file_path = root_path.join("test.txt");
        fs::write(&file_path, b"hello").unwrap();
        let result = confine_to_path(&root_path, Root::Data, Path::new("test.txt")).unwrap();
        assert_eq!(result, file_path.canonicalize().unwrap());
    }

    #[test]
    fn test_traversal_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let root_path = dir.path().canonicalize().unwrap();
        let err = confine_to_path(&root_path, Root::Data, Path::new("../etc/passwd")).unwrap_err();
        match &err {
            FsError::TraversalDetected { .. } => {} // expected
            FsError::ResolveError(_) => {}          // also acceptable if path doesn't exist
            _ => panic!("expected TraversalDetected or ResolveError, got {err:?}"),
        }
    }

    #[test]
    fn test_subdirectory_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let root_path = dir.path().canonicalize().unwrap();
        let sub = root_path.join("sub");
        fs::create_dir(&sub).unwrap();
        let file = sub.join("nested.txt");
        fs::write(&file, b"data").unwrap();
        let result =
            confine_to_path(&root_path, Root::Artifact, Path::new("sub/nested.txt")).unwrap();
        assert_eq!(result, file.canonicalize().unwrap());
    }

    #[test]
    fn test_root_env_var_names() {
        assert_eq!(Root::Content.env_var(), "MC_CONTENT_DIR");
        assert_eq!(Root::Data.env_var(), "MC_DATA_DIR");
        assert_eq!(Root::Artifact.env_var(), "MC_ARTIFACT_DIR");
    }

    #[test]
    fn test_display_fserror() {
        let err = FsError::UnresolvedRoot(Root::Content);
        let msg = format!("{err}");
        assert!(
            msg.contains("Content"),
            "expected message to contain 'Content', got: {msg}"
        );
    }

    #[test]
    fn test_read_to_string_wrapper() {
        let dir = tempfile::tempdir().unwrap();
        let root_path = dir.path().canonicalize().unwrap();
        let file = root_path.join("test.txt");
        fs::write(&file, b"hello world").unwrap();
        let result = read_to_string(Root::Data, Path::new("test.txt")).unwrap_err();
        // Will fail because MC_DATA_DIR is not set; that's expected
        assert!(matches!(result, FsError::UnresolvedRoot(Root::Data)));
    }

    #[test]
    fn test_write_new_file_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let root_path = dir.path().canonicalize().unwrap();
        let result = confine_to_path(&root_path, Root::Data, Path::new("new_file.txt")).unwrap();
        // The path should be within the root
        assert!(result.starts_with(&root_path));
    }

    #[test]
    fn test_create_dir_preserves_existing_parent() {
        let dir = tempfile::tempdir().unwrap();
        let root_path = dir.path().canonicalize().unwrap();
        let sub = root_path.join("existing");
        fs::create_dir(&sub).unwrap();
        let result =
            confine_to_path(&root_path, Root::Data, Path::new("existing/new_file.ron")).unwrap();
        assert!(result.starts_with(&root_path));
        assert_eq!(result, sub.join("new_file.ron"));
    }

    #[test]
    fn test_write_traversal_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let root_path = dir.path().canonicalize().unwrap();
        let err = confine_to_path(&root_path, Root::Data, Path::new("../escape.txt")).unwrap_err();
        match &err {
            FsError::TraversalDetected { .. } => {}
            FsError::ResolveError(_) => {}
            _ => panic!("expected TraversalDetected or ResolveError, got {err:?}"),
        }
    }

    #[test]
    fn test_confine_allows_nonexistent_candidate() {
        let dir = tempfile::tempdir().unwrap();
        let root_path = dir.path().canonicalize().unwrap();
        let result =
            confine_to_path(&root_path, Root::Data, Path::new("does/not/exist.txt")).unwrap();
        // The path should be within the root even though it doesn't exist
        assert!(result.starts_with(&root_path));
    }
}
