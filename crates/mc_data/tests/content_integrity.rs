//! Content integrity tests for the bake pipeline.
//!
//! Verifies that:
//! - The pipeline runs cleanly against the full content tree
//! - The pack is deterministic (identical on two builds)
//! - Save/load round-trips correctly
//! - All cross-file references resolve
//! - Corrupted digests are detected

use std::path::PathBuf;
use std::fs;
use mc_data::pack::Pack;

fn content_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); p.pop(); // repo root
    p.join("content")
}

fn tmp_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); p.pop();
    p.join("target").join("tmp-test-pack")
}

fn require_clean_bake() {
    let root = content_root();
    let errors = mc_data::bake::bake(&root);
    assert!(errors.is_ok(), "content validation must pass before pack test");
}

fn build_pack() -> Pack {
    require_clean_bake();
    Pack::from_content(&content_root()).expect("should build pack from clean content")
}

/// Deterministic: two builds produce identical bytes.
#[test]
fn bake_pack_is_deterministic() {
    let pack_a = build_pack();
    let pack_b = build_pack();
    assert_eq!(pack_a.to_bytes(), pack_b.to_bytes(), "pack bytes must be identical");
    assert_eq!(pack_a.digest(), pack_b.digest(), "pack digest must be identical");
}

/// Save then load produces an identical pack.
#[test]
fn save_load_roundtrip() {
    let pack = build_pack();
    let tmp = tmp_dir();
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    pack.save(&tmp.join("content.pack")).unwrap();

    let loaded = Pack::load_from_dir(&tmp).unwrap_or_else(|_| {
        // Hack: if load_from_dir fails, try finding blake3 as content.pack.blake3
        Pack::load(
            &tmp.join("content.pack"),
            &tmp.join("content.pack.blake3"),
        ).expect("should load saved pack")
    });

    assert_eq!(pack.to_bytes(), loaded.to_bytes(), "round-trip pack must match");
    assert_eq!(pack.digest(), loaded.digest(), "digest must match after round-trip");

    let _ = fs::remove_dir_all(&tmp);
}

/// Baking twice produces identical digests.
#[test]
fn bake_twice_identical_digests() {
    let pack = build_pack();
    let tmp = tmp_dir();
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    let d1 = pack.digest();
    pack.save(&tmp.join("content.pack")).unwrap();

    let loaded = Pack::load_from_dir(&tmp).unwrap();
    let d2 = loaded.digest();

    assert_eq!(d1, d2, "bake-twice digests should be identical");

    let _ = fs::remove_dir_all(&tmp);
}

/// All cross-file references in the pack resolve.
#[test]
fn all_references_resolve() {
    let pack = build_pack();
    let errors = mc_data::pack::verify_references(&pack);
    if !errors.is_empty() {
        for err in &errors {
            eprintln!("REFERENCE ERROR: {err}");
        }
        panic!("{} unresolved reference(s) found in pack", errors.len());
    }
}

/// Spawn table enemy IDs exist in bestiary.
#[test]
fn bestiary_has_no_missing_enemy_ids() {
    let pack = build_pack();
    let enemy_ids: Vec<&str> = pack.enemies.iter().map(|e| e.id.as_str()).collect();

    for table in &pack.spawn_tables {
        for entry in &table.entries {
            assert!(
                enemy_ids.contains(&entry.enemy.as_str()),
                "spawn table {} references enemy {} not in bestiary",
                table.region, entry.enemy
            );
        }
    }
}

/// Region connections exist.
#[test]
fn region_connections_all_exist() {
    let pack = build_pack();
    let region_ids: Vec<&str> = pack.regions.iter().map(|r| r.id.as_str()).collect();

    for region in &pack.regions {
        for conn in &region.connections {
            let found = region_ids.iter().any(|id| *id == conn);
            assert!(
                found,
                "region `{}` connects to `{conn}` but no region with that ID exists",
                region.id
            );
        }
    }
}

/// Scene item references resolve.
#[test]
fn scene_item_references_resolve() {
    let pack = build_pack();
    let item_ids: Vec<&str> = pack.items.iter().map(|i| i.id.as_str()).collect();

    for scene in &pack.scenes {
        if let Some(ref effects) = scene.on_exit {
            for grant_id in &effects.grant {
                let found = item_ids.iter().any(|id| *id == grant_id);
                assert!(
                    found,
                    "scene `{}` grants item `{grant_id}` but no item with that ID exists",
                    scene.id
                );
            }
            for consume_id in &effects.consume {
                let found = item_ids.iter().any(|id| *id == consume_id);
                assert!(
                    found,
                    "scene `{}` consumes item `{consume_id}` but no item with that ID exists",
                    scene.id
                );
            }
        }
    }
}

/// Enemy loot references resolve.
#[test]
fn enemy_loot_references_resolve() {
    let pack = build_pack();
    let item_ids: Vec<&str> = pack.items.iter().map(|i| i.id.as_str()).collect();

    for enemy in &pack.enemies {
        for entry in &enemy.loot {
            let loot_id = &entry.0;
            let found = item_ids.iter().any(|id| *id == loot_id);
            assert!(
                found,
                "enemy `{}` drops item `{loot_id}` but no item with that ID exists",
                enemy.id
            );
        }
    }
}

/// All tracked counts are present after build.
#[test]
fn pack_has_content() {
    let pack = build_pack();
    assert!(!pack.enemies.is_empty(), "pack must have enemies");
    assert!(!pack.regions.is_empty(), "pack must have regions");
    assert!(!pack.scenes.is_empty(), "pack must have scenes");
    assert!(!pack.spawn_tables.is_empty(), "pack must have spawn tables");
    assert!(!pack.items.is_empty(), "pack must have items");
    assert!(!pack.flags.is_empty(), "pack must have flags");
}

/// Loading with corrupted digest fails.
#[test]
fn pack_load_rejects_corrupted_digest() {
    let pack = build_pack();
    let tmp = tmp_dir();
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    pack.save(&tmp.join("content.pack")).unwrap();

    let digest_path = tmp.join("content.pack.blake3");
    fs::write(&digest_path, "INVALID_HEX_DIGEST_HERE\n").unwrap();

    let result = Pack::load_from_dir(&tmp);
    assert!(result.is_err(), "loading with corrupted digest should fail");

    let _ = fs::remove_dir_all(&tmp);
}

/// Loading with missing files fails.
#[test]
fn pack_load_rejects_missing_file() {
    let tmp = tmp_dir();
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    let result = Pack::load_from_dir(&tmp);
    assert!(result.is_err(), "loading from empty dir should fail");

    let _ = fs::remove_dir_all(&tmp);
}
