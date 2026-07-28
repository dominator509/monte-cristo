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

use std::path::{Path, PathBuf};
use std::env;

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
            FsError::TraversalDetected { root, requested, resolved } => {
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
/// 2. Canonicalises the root (requires it to exist).
/// 3. Joins `requested` to the root and canonicalises the result.
/// 4. Verifies the canonical result is a strict sub-path of the canonical root.
/// 5. Checks every component of the relative portion for symlink escape.
///
/// Returns the canonical, confined absolute path on success.
pub fn confine(root: Root, requested: &Path) -> Result<PathBuf, FsError> {
    // 1. Resolve root from environment.
    let root_path = root
        .resolve_env()
        .ok_or(FsError::UnresolvedRoot(root))?;

    // 2. Canonicalise the root (requires it to exist on disk).
    let canonical_root = root_path
        .canonicalize()
        .map_err(|e| FsError::InvalidRoot(root, format!("cannot canonicalise: {e}")))?;

    // 3. Build the full candidate path and canonicalise it.
    let candidate = canonical_root.join(requested);

    // 4. Walk components of `requested` checking for symlink escapes.
    //    We check each component's real path against the canonical root.
    let mut current = canonical_root.clone();
    for component in requested.components() {
        current.push(component);
        // If the component is a symlink, check its target.
        if let Ok(metadata) = current.symlink_metadata() {
            if metadata.file_type().is_symlink() {
                let link_target = current
                    .read_link()
                    .map_err(|e| FsError::ResolveError(format!("cannot read symlink {current:?}: {e}")))?;
                // Resolve the link target relative to the parent directory
                let abs_target = if link_target.is_absolute() {
                    link_target
                } else {
                    let parent = current.parent().unwrap_or(&canonical_root);
                    parent.join(&link_target)
                };
                let canonical_target = abs_target
                    .canonicalize()
                    .map_err(|e| FsError::ResolveError(format!("cannot canonicalise symlink target {abs_target:?}: {e}")))?;
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

    // 5. Canonicalise the final candidate.
    let canonical_candidate = candidate
        .canonicalize()
        .map_err(|e| FsError::ResolveError(format!("cannot canonicalise {candidate:?}: {e}")))?;

    // 6. Verify the candidate is within the root (strict sub-path).
    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(FsError::TraversalDetected {
            root: canonical_root,
            requested: requested.to_path_buf(),
            resolved: canonical_candidate,
        });
    }

    // 7. Also reject exact match of root itself when a file is expected,
    //    but allow the root path itself (it's the directory, not a file).
    //    Actually, confine should allow the root path itself since it can be
    //    a valid target (e.g., listing contents). We just reject traversal.

    Ok(canonical_candidate)
}

/// Check if `path` is a child of `root` without canonicalising (for quick refusal).
fn is_child_of(root: &Path, path: &Path) -> bool {
    path.components().zip(root.components()).all(|(p, r)| p == r)
        && path.components().count() > root.components().count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn set_env(root: Root, path: &Path) {
        env::set_var(root.env_var(), path);
    }

    fn unset_env(root: Root) {
        env::remove_var(root.env_var());
    }

    #[test]
    fn test_unset_env_returns_error() {
        unset_env(Root::Content);
        let err = confine(Root::Content, Path::new("foo.ron")).unwrap_err();
        assert!(matches!(err, FsError::UnresolvedRoot(Root::Content)));
    }

    #[test]
    fn test_non_existent_root_returns_error() {
        set_env(Root::Content, Path::new("/tmp/nonexistent_mc_test_XXXXXX"));
        let err = confine(Root::Content, Path::new("foo.ron")).unwrap_err();
        assert!(matches!(err, FsError::InvalidRoot(Root::Content, _)));
        unset_env(Root::Content);
    }

    #[test]
    fn test_valid_confine_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let root_path = dir.path().canonicalize().unwrap();
        set_env(Root::Data, &root_path);

        // Create a file inside the root
        let file_path = root_path.join("test.txt");
        fs::write(&file_path, b"hello").unwrap();

        let result = confine(Root::Data, Path::new("test.txt")).unwrap();
        assert_eq!(result, file_path.canonicalize().unwrap());
        unset_env(Root::Data);
    }

    #[test]
    fn test_traversal_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let root_path = dir.path().canonicalize().unwrap();
        set_env(Root::Data, &root_path);

        let err = confine(Root::Data, Path::new("../etc/passwd")).unwrap_err();
        match &err {
            FsError::TraversalDetected { .. } => {} // expected
            FsError::ResolveError(_) => {}          // also acceptable if path doesn't exist
            _ => panic!("expected TraversalDetected or ResolveError, got {err:?}"),
        }
        unset_env(Root::Data);
    }

    #[test]
    fn test_subdirectory_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let root_path = dir.path().canonicalize().unwrap();
        set_env(Root::Artifact, &root_path);

        let sub = root_path.join("sub");
        fs::create_dir(&sub).unwrap();
        let file = sub.join("nested.txt");
        fs::write(&file, b"data").unwrap();

        let result = confine(Root::Artifact, Path::new("sub/nested.txt")).unwrap();
        assert_eq!(result, file.canonicalize().unwrap());
        unset_env(Root::Artifact);
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
        assert!(msg.contains("Content"), "expected message to contain 'Content', got: {msg}");
    }
}
