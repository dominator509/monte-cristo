//! M6: Save round-trip tests.
//!
//! Verifies that a Save can be serialised → deserialised with identical
//! world state, that file I/O works, and that bounds are enforced.

use std::fs;
use std::path::PathBuf;

use mc_core::world::World;
use mc_data::pack::Pack;
use mc_data::save::{Save, CURRENT_SCHEMA_VERSION, MAX_PRODUCT_VERSION_LEN, MAX_SCHEMA_VERSION};

fn content_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop(); // repo root
    p.join("content")
}

fn tmp_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p.join("target").join("tmp-test-save-roundtrip")
}

fn build_content_digest() -> [u8; 32] {
    let pack = Pack::from_content(&content_root()).expect("pack should build from content");
    *blake3::hash(&pack.to_bytes()).as_bytes()
}

/// Build a save, round-trip it through bytes, and assert state_hash matches.
#[test]
fn save_roundtrip_preserves_world() {
    let content_digest = build_content_digest();

    let mut world = World::new(42);
    for _ in 0..10 {
        world.step();
    }
    let original_hash = world.state_hash();

    let save = Save::new(
        CURRENT_SCHEMA_VERSION,
        "0.1.0".to_string(),
        content_digest,
        world,
    );

    let bytes = save.to_bytes();
    let loaded = Save::load(&bytes).expect("should deserialise save");
    let roundtrip_hash = loaded.world.state_hash();

    assert_eq!(
        original_hash, roundtrip_hash,
        "world state hash must match after save round-trip"
    );
    assert_eq!(save.schema_version, loaded.schema_version);
    assert_eq!(save.product_version, loaded.product_version);
    assert_eq!(save.content_digest, loaded.content_digest);
}

/// Save to file, load from file, assert state matches.
#[test]
fn save_file_roundtrip() {
    let _ = fs::remove_dir_all(&tmp_dir());
    fs::create_dir_all(&tmp_dir()).unwrap();

    let content_digest = build_content_digest();
    let mut world = World::new(42);
    for _ in 0..10 {
        world.step();
    }
    let original_hash = world.state_hash();

    let save = Save::new(
        CURRENT_SCHEMA_VERSION,
        "0.1.0".to_string(),
        content_digest,
        world,
    );

    let save_path = tmp_dir().join("test.sav");
    save.to_file(&save_path).expect("save to file");

    let loaded = Save::from_file(&save_path).expect("load from file");
    assert_eq!(
        original_hash,
        loaded.world.state_hash(),
        "world state must match after file round-trip"
    );

    let _ = fs::remove_dir_all(&tmp_dir());
}

/// Reject schema_version beyond the maximum.
#[test]
fn reject_huge_schema_version() {
    let content_digest = build_content_digest();
    let world = World::new(42);

    let huge: u16 = MAX_SCHEMA_VERSION + 1;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        Save::new(huge, "0.1.0".into(), content_digest, world);
    }));
    assert!(result.is_err(), "should panic on huge schema version");
}

/// Reject product_version longer than limit.
#[test]
fn reject_long_product_version() {
    let content_digest = build_content_digest();
    let world = World::new(42);

    let long_product = "a".repeat(MAX_PRODUCT_VERSION_LEN + 1);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        Save::new(CURRENT_SCHEMA_VERSION, long_product, content_digest, world);
    }));
    assert!(result.is_err(), "should panic on long product version");
}

/// Reject save files with newer schema version.
#[test]
fn reject_newer_save_version() {
    let content_digest = build_content_digest();
    let world = World::new(42);

    // Build a save with a version higher than CURRENT
    // We can't use Save::new because it asserts. Build it manually.
    let mut save = Save {
        schema_version: CURRENT_SCHEMA_VERSION + 1,
        product_version: "0.1.0".into(),
        content_digest,
        world,
        digest: [0u8; 32],
    };
    let bytes = save.to_bytes();

    let result = Save::load(&bytes);
    assert!(
        result.is_err(),
        "loading a newer-version save should be rejected"
    );
}

/// Reject corrupted saves.
#[test]
fn reject_corrupted_digest() {
    let content_digest = build_content_digest();
    let mut world = World::new(42);
    world.step();

    let save = Save::new(
        CURRENT_SCHEMA_VERSION,
        "0.1.0".into(),
        content_digest,
        world,
    );
    let mut bytes = save.to_bytes();
    // Flip one byte in the body
    let flip_pos = bytes.len() / 2;
    bytes[flip_pos] ^= 0x01;

    let result = Save::load(&bytes);
    assert!(
        matches!(
            result,
            Err(mc_data::error::SaveError::DigestMismatch { .. })
        ),
        "corrupted save should produce DigestMismatch, got {result:?}"
    );
}
