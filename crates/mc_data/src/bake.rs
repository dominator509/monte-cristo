//! The content bake pipeline.
//!
//! Orchestrates all seven validators from [`crate::validate`] in the order
//! specified by SPEC-002 section 2. Each validator runs; if any returns
//! non-empty errors, the pipeline short-circuits and returns the first
//! batch of errors found.

use std::path::Path;

use crate::error::ContentError;
use crate::validate;

/// Run the full bake pipeline over the content directory at `content_root`.
///
/// Validators execute in SPEC-002 order:
/// 1. `schema_check`
/// 2. `vocabulary_check`
/// 3. `reference_resolve`
/// 4. `orphan_detect`
/// 5. `supernatural_lint`
/// 6. `region_affinity_check`
/// 7. `reserved_identifier_reject`
///
/// Returns `Ok(())` if all validators pass. Returns the errors from the
/// first validator that fails (short-circuit).
pub fn bake(content_root: &Path) -> Result<(), Vec<ContentError>> {
    // 1 — schema_check
    let errors = validate::schema_check(content_root);
    if !errors.is_empty() {
        return Err(errors);
    }

    // 2 — vocabulary_check
    let errors = validate::vocabulary_check(content_root);
    if !errors.is_empty() {
        return Err(errors);
    }

    // 3 — reference_resolve
    let errors = validate::reference_resolve(content_root);
    if !errors.is_empty() {
        return Err(errors);
    }

    // 4 — orphan_detect
    let errors = validate::orphan_detect(content_root);
    if !errors.is_empty() {
        return Err(errors);
    }

    // 5 — supernatural_lint
    let errors = validate::supernatural_lint(content_root);
    if !errors.is_empty() {
        return Err(errors);
    }

    // 6 — region_affinity_check
    let errors = validate::region_affinity_check(content_root);
    if !errors.is_empty() {
        return Err(errors);
    }

    // 7 — reserved_identifier_reject
    let errors = validate::reserved_identifier_reject(content_root);
    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(())
}

/// Convenience wrapper that resolves the content directory relative to
/// `CARGO_MANIFEST_DIR` (i.e. relative to the `mc_data` crate root).
///
/// In the monte-cristo workspace this resolves to `<repo>/content/`.
pub fn bake_from_manifest_dir() -> Result<(), Vec<ContentError>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    // mc_data lives at crates/mc_data; content is at repo_root/content/
    let content_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(|p| p.join("content"))
        .unwrap_or_else(|| manifest_dir.join("../../content"));

    bake(&content_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn bake_pipeline_runs_without_panicking() {
        // Locate content relative to this test's run dir
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let content_root = manifest_dir
            .parent()
            .and_then(Path::parent)
            .map(|p| p.join("content"))
            .expect("content directory should exist");
        assert!(
            content_root.exists(),
            "content dir should exist at: {}",
            content_root.display()
        );

        let result = bake(&content_root);
        // Print all errors for human inspection
        if let Err(errors) = &result {
            for err in errors {
                eprintln!("BAKE ERROR: {err}");
            }
        }
        // The pipeline should always run without panicking,
        // even if the content has unresolved references / missing files.
        // Validators report errors; they don't crash.
    }
}
