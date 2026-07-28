//! Save-file migration layer.
//!
//! M7: Supports one-shot migration from v1 → current.
//!
//! On migration the original file is renamed to `<name>.v<N-1>.bak` so that
//! the original data is never deleted.

use std::fs;
use std::path::Path;

use mc_core::world::World;

use crate::error::SaveError;
use crate::save::{Save, CURRENT_SCHEMA_VERSION};

/// The schema version recognised by the v1 parser.
const V1_SCHEMA_VERSION: u16 = 1;

/// Migrate a v1 save blob into the current format.
///
/// `data` is the raw bytes of a v1 save.  v1 format (postcard tuple):
///
///   `(schema_version: u16, product_version: &str, content_digest: [u8; 32], world: &World)`
///
/// No trailing integrity digest existed in v1.
pub fn migrate_save(data: &[u8]) -> Result<Vec<u8>, SaveError> {
    // Decode v1 format
    let (schema_version, product_version, content_digest, world): (u16, String, [u8; 32], World) =
        postcard::from_bytes(data)
            .map_err(|e| SaveError::Deserialize(format!("v1 decode failed: {e}")))?;

    if schema_version != V1_SCHEMA_VERSION {
        return Err(SaveError::Deserialize(format!(
            "expected v1 (schema_version={V1_SCHEMA_VERSION}), got {schema_version}"
        )));
    }

    // Build a current-format Save (digest is computed in new())
    let save = Save::new(
        CURRENT_SCHEMA_VERSION,
        product_version,
        content_digest,
        world,
    )?;

    Ok(save.to_bytes()?)
}

/// Migrate a v1 save file on disk.
///
/// The original file is renamed from `<path>` to `<path>.v1.bak`, and the
/// migrated content is written to `<path>`.  The original is never deleted.
pub fn migrate_save_file(path: &Path) -> Result<(), SaveError> {
    // Build backup name
    let backup = path.with_extension("v1.bak");

    // Read the original data
    let data = fs::read(path)?;

    // Migrate
    let migrated = migrate_save(&data)?;

    // Rename original → backup (never delete)
    fs::rename(path, &backup)?;

    // Write new save
    fs::write(path, &migrated)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::Pack;
    use crate::save::Save;
    use std::path::PathBuf;

    fn content_root() -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop();
        p.pop(); // repo root
        p.join("content")
    }

    fn build_v1_bytes() -> Vec<u8> {
        let root = content_root();
        let pack = Pack::from_content(&root).expect("pack should build");
        let content_digest = *blake3::hash(&pack.to_bytes().unwrap()).as_bytes();

        let world = World::new(42);

        postcard::to_stdvec(&(V1_SCHEMA_VERSION, "0.1.0", content_digest, &world))
            .expect("v1 serialization should not fail")
    }

    #[test]
    fn migrate_v1_to_current() {
        let v1_bytes = build_v1_bytes();
        let migrated = migrate_save(&v1_bytes).expect("migration should succeed");

        let save = Save::load(&migrated).expect("migrated save should load");
        assert_eq!(save.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(save.product_version, "0.1.0");
    }

    #[test]
    fn migrate_rejects_bad_version() {
        let v1_bytes = build_v1_bytes();
        // Patch schema version to 3
        let _current_version = v1_bytes[0] as u16 | (v1_bytes[1] as u16) << 8;
        let mut bad = v1_bytes.clone();
        let patched: u16 = 99; // wrong version
        bad[0] = patched as u8;
        bad[1] = (patched >> 8) as u8;
        let result = migrate_save(&bad);
        assert!(result.is_err(), "wrong version should be rejected");
    }
}
