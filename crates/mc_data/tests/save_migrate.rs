//! M7: Save migration tests.
//!
//! Verifies that a v1 save file can be migrated to the current format,
//! that content_digest is preserved, and that a .bak file is created.

use std::fs;
use std::path::PathBuf;

use mc_core::world::World;
use mc_data::migrate;
use mc_data::pack::Pack;
use mc_data::save::{Save, CURRENT_SCHEMA_VERSION};

fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop(); // repo root
    p
}

fn fixture_v1_path() -> PathBuf {
    repo_root()
        .join("tests")
        .join("fixtures")
        .join("saves-v1")
        .join("save.v1")
}

fn tmp_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p.join("target").join("tmp-test-save-migrate")
}

/// Load the v1 fixture, migrate it in memory, and verify the result.
#[test]
fn migrate_v1_fixture_in_memory() {
    let data = fs::read(fixture_v1_path())
        .unwrap_or_else(|e| panic!("cannot read v1 fixture at {:?}: {e}", fixture_v1_path()));

    let migrated = migrate::migrate_save(&data).expect("migration should succeed");

    let save = Save::load(&migrated).expect("migrated save should load");
    assert_eq!(
        save.schema_version, CURRENT_SCHEMA_VERSION,
        "migrated save should have current schema version"
    );
    assert_eq!(
        save.product_version, "0.1.0",
        "product_version should be preserved"
    );
    assert!(
        save.content_digest != [0u8; 32],
        "content_digest should be preserved (non-zero)"
    );
}

/// Migrate the fixture file on disk and verify the .bak file was created.
#[test]
fn migrate_v1_fixture_file() {
    let _ = fs::remove_dir_all(tmp_dir());
    fs::create_dir_all(tmp_dir()).unwrap();

    // Copy the v1 fixture to our temp dir
    let save_path = tmp_dir().join("save.sav");
    fs::copy(fixture_v1_path(), &save_path).unwrap_or_else(|e| panic!("cannot copy fixture: {e}"));

    // Read original data for comparison
    let original_data = fs::read(&save_path).expect("should read copied fixture");

    // Migrate the file in place
    migrate::migrate_save_file(&save_path).expect("file migration should succeed");

    // Check .bak file exists
    let bak_path = tmp_dir().join("save.v1.bak");
    assert!(
        bak_path.exists(),
        "backup file {:?} should exist after migration",
        bak_path
    );

    // The backup should match the original v1 data
    let bak_data = fs::read(&bak_path).expect("should read backup");
    assert_eq!(
        bak_data, original_data,
        "backup must contain original v1 data"
    );

    // The new save file should load correctly
    let save = Save::from_file(&save_path).expect("migrated save should load from file");
    assert_eq!(save.schema_version, CURRENT_SCHEMA_VERSION);
    assert_eq!(save.product_version, "0.1.0");

    let _ = fs::remove_dir_all(tmp_dir());
}

/// Verifying that migration preserves world state_hash.
#[test]
fn migrate_preserves_world_state() {
    // Build a v1 save from a known world
    let content_digest = {
        let root = repo_root().join("content");
        let pack = Pack::from_content(&root).expect("pack should build");
        *blake3::hash(&pack.to_bytes().unwrap()).as_bytes()
    };

    let mut world = World::new(42);
    for _ in 0..5 {
        world.step();
    }
    let original_hash = world.state_hash();

    let v1_bytes =
        postcard::to_stdvec(&(1u16, "0.1.0", content_digest, &world)).expect("v1 serialization");

    let migrated = migrate::migrate_save(&v1_bytes).expect("migration should succeed");
    let save = Save::load(&migrated).expect("migrated save should load");

    assert_eq!(
        original_hash,
        save.world.state_hash(),
        "world state_hash must be preserved after migration"
    );
    assert_eq!(save.content_digest, content_digest);
}
