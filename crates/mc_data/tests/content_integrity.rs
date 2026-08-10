//! Content integrity tests for the bake pipeline.
//!
//! Verifies that:
//! - The pipeline runs cleanly against the full content tree
//! - The pack is deterministic (identical on two builds)
//! - Save/load round-trips correctly
//! - All cross-file references resolve
//! - Corrupted digests are detected

use mc_core::command::{apply_commands_with_catalog, ChoiceIdx, Command, CoreEvent};
use mc_core::ids::FlagId;
use mc_core::world::World;
use mc_data::pack::Pack;
use std::fs;
use std::path::PathBuf;

fn content_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop(); // repo root
    p.join("content")
}

fn tmp_dir(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p.join("target").join(format!("tmp-test-pack-{}", name))
}

fn require_clean_bake() {
    let root = content_root();
    let errors = mc_data::bake::bake(&root);
    assert!(
        errors.is_ok(),
        "content validation must pass before pack test"
    );
}

fn build_pack() -> Pack {
    require_clean_bake();
    Pack::from_content(&content_root()).expect("should build pack from clean content")
}

#[test]
fn pack_includes_every_authored_scene_act() {
    let pack = build_pack();
    let scenes_root = content_root().join("scenes");
    let expected = fs::read_dir(&scenes_root)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|act| {
            fs::read_dir(act.path())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "ron"))
                .count()
        })
        .sum::<usize>();

    assert_eq!(pack.scenes.len(), expected);
    assert!(pack
        .scenes
        .iter()
        .any(|scene| scene.id == "SCN_CONFIDENCE_CF45"));
}

#[test]
fn authored_arrest_scene_executes_through_core_catalog() {
    let pack = build_pack();
    let catalog = pack
        .scene_catalog()
        .expect("authored scenes should convert to the core catalog");
    assert_eq!(catalog.scene_count(), pack.scenes.len());
    assert!(catalog.node_count() >= catalog.scene_count());

    let mut world = World::new(42);
    catalog
        .begin(&mut world, "SCN_ARREST")
        .expect("arrest scene should be available at new game");
    for choice in [0, 0] {
        let events = apply_commands_with_catalog(
            &mut world,
            &[Command::SceneChoose(ChoiceIdx(choice))],
            Some(&catalog),
        );
        assert!(matches!(events[0], CoreEvent::Applied { .. }));
    }
    let events = apply_commands_with_catalog(&mut world, &[Command::SceneAdvance], Some(&catalog));
    assert!(matches!(events[0], CoreEvent::Applied { .. }));
    assert!(world.scene.is_none());
    assert!(world.flags.is_set(FlagId::FLG_ARRESTED));
}

#[test]
fn authored_item_catalog_preserves_consumable_and_key_semantics() {
    let pack = build_pack();
    let catalog = pack
        .item_catalog()
        .expect("authored items should convert to the core catalog");
    assert_eq!(catalog.len(), pack.items.len());
    assert_eq!(
        catalog
            .get(mc_core::ids::ItemId::ITM_POTION)
            .expect("potion should be authored")
            .heal_hp,
        Some(25)
    );
    assert_eq!(
        catalog
            .get(mc_core::ids::ItemId::ITM_TREASURE_MAP)
            .expect("treasure map should be authored")
            .kind,
        mc_core::item::ItemKind::Key
    );
}

/// Deterministic: two builds produce identical bytes.
#[test]
fn bake_pack_is_deterministic() {
    let pack_a = build_pack();
    let pack_b = build_pack();
    assert_eq!(
        pack_a.to_bytes().unwrap(),
        pack_b.to_bytes().unwrap(),
        "pack bytes must be identical"
    );
    assert_eq!(
        pack_a.digest().unwrap(),
        pack_b.digest().unwrap(),
        "pack digest must be identical"
    );
}

/// Save then load produces an identical pack.
#[test]
fn save_load_roundtrip() {
    let pack = build_pack();
    let tmp = tmp_dir("roundtrip");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    pack.save(&tmp.join("content.pack")).unwrap();

    let loaded = Pack::load_from_dir(&tmp).unwrap_or_else(|_| {
        // Hack: if load_from_dir fails, try finding blake3 as content.pack.blake3
        Pack::load(&tmp.join("content.pack"), &tmp.join("content.pack.blake3"))
            .expect("should load saved pack")
    });

    assert_eq!(
        pack.to_bytes().unwrap(),
        loaded.to_bytes().unwrap(),
        "round-trip pack must match"
    );
    assert_eq!(
        pack.digest().unwrap(),
        loaded.digest().unwrap(),
        "digest must match after round-trip"
    );

    let _ = fs::remove_dir_all(&tmp);
}

/// Baking twice produces identical digests.
#[test]
fn bake_twice_identical_digests() {
    let pack = build_pack();
    let tmp = tmp_dir("bake_twice");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    let d1 = pack.digest().unwrap();
    pack.save(&tmp.join("content.pack")).unwrap();

    let loaded = Pack::load_from_dir(&tmp).unwrap();
    let d2 = loaded.digest().unwrap();

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
                table.region,
                entry.enemy
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
    let tmp = tmp_dir("corrupted");
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
    let tmp = tmp_dir("missing");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    let result = Pack::load_from_dir(&tmp);
    assert!(result.is_err(), "loading from empty dir should fail");

    let _ = fs::remove_dir_all(&tmp);
}
