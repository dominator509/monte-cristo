//! M7: Backup/restore tests.
//!
//! Verifies that a corrupt save file is rejected with DigestMismatch while
//! an intact copy still loads correctly.

use std::fs;
use std::path::PathBuf;

use mc_core::world::World;
use mc_data::error::SaveError;
use mc_data::pack::Pack;
use mc_data::save::Save;

fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p
}

fn tmp_dir(label: &str) -> PathBuf {
    repo_root()
        .join("target")
        .join(format!("tmp-test-backup-restore-{label}"))
}

fn build_save() -> Save {
    let root = repo_root().join("content");
    let pack = Pack::from_content(&root).expect("pack should build");
    let content_digest = *blake3::hash(&pack.to_bytes()).as_bytes();

    let mut world = World::new(42);
    for _ in 0..10 {
        world.step();
    }

    Save::new(2, "0.1.0".into(), content_digest, world)
}

#[test]
fn corrupt_file_rejected_with_digest_mismatch() {
    let _ = fs::remove_dir_all(&tmp_dir("corrupt"));
    fs::create_dir_all(&tmp_dir("corrupt")).unwrap();

    let save = build_save();
    let save_path = tmp_dir("corrupt").join("save.sav");
    save.to_file(&save_path).expect("save to file");

    // Copy the save to a "corrupt" copy
    let corrupt_path = tmp_dir("corrupt").join("save_corrupt.sav");
    fs::copy(&save_path, &corrupt_path).expect("copy save");

    // Flip one byte in the trailing digest (last 32 bytes)
    let mut data = fs::read(&corrupt_path).expect("read copy");
    let data_len = data.len();
    let flip_pos = data_len - 1; // last byte of the digest
    data[flip_pos] ^= 0xFF;
    fs::write(&corrupt_path, &data).expect("write corrupted data");

    // Assert corrupt file is rejected with DigestMismatch
    let result = Save::from_file(&corrupt_path);
    assert!(
        matches!(result, Err(SaveError::DigestMismatch { .. })),
        "corrupt save should produce DigestMismatch, got {result:?}"
    );
    // Also verify the type by matching
    match result {
        Err(SaveError::DigestMismatch { expected, actual }) => {
            assert!(!expected.is_empty(), "expected digest should not be empty");
            assert!(!actual.is_empty(), "actual digest should not be empty");
        }
        other => panic!("expected DigestMismatch, got {other:?}"),
    }

    // Assert intact file still loads
    let loaded = Save::from_file(&save_path).expect("intact save should load");
    assert_eq!(loaded.schema_version, 2);
    assert_eq!(
        loaded.world.state_hash(),
        save.world.state_hash(),
        "intact save must preserve world state"
    );

    let _ = fs::remove_dir_all(&tmp_dir("corrupt"));
}

#[test]
fn intact_file_loads_after_copy() {
    let _ = fs::remove_dir_all(&tmp_dir("intact"));
    fs::create_dir_all(&tmp_dir("intact")).unwrap();

    let save = build_save();
    let save_path = tmp_dir("intact").join("save.sav");
    save.to_file(&save_path).expect("save to file");

    // Copy to another location
    let copy_path = tmp_dir("intact").join("save_copy.sav");
    fs::copy(&save_path, &copy_path).expect("copy save");

    // Load from copy
    let loaded = Save::from_file(&copy_path).expect("copied save should load");
    assert_eq!(
        save.world.state_hash(),
        loaded.world.state_hash(),
        "copied save must preserve world state"
    );

    let _ = fs::remove_dir_all(&tmp_dir("intact"));
}
