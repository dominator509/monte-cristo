//! Forced failure mode tests for the save/load pipeline.
//!
//! Each test exercises a real failure mode — no simulated conditions.

use std::fs;
use std::path::{Path, PathBuf};

use mc_core::world::World;
use mc_data::error::SaveError;
use mc_data::pack::Pack;
use mc_data::save::Save;

// ---------------------------------------------------------------------------
// Helpers (mirror backup_restore.rs)
// ---------------------------------------------------------------------------

fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p
}

fn tmp_dir(label: &str) -> PathBuf {
    repo_root()
        .join("target")
        .join(format!("tmp-test-forced-failures-{label}"))
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

fn build_pack() -> Pack {
    let root = repo_root().join("content");
    Pack::from_content(&root).expect("pack should build from real content")
}

// ===========================================================================
// 1. truncated_file
// ===========================================================================

#[test]
fn truncated_file() {
    let tmp = tmp_dir("truncated");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).expect("create tmp dir");

    let save = build_save();
    let path = tmp.join("save.sav");
    save.to_file(&path).expect("save to file");

    // Truncate to half its original size
    let data = fs::read(&path).expect("read save");
    let half_len = data.len() / 2;
    fs::write(&path, &data[..half_len]).expect("truncate file");

    // Loading must return an Err, not panic
    let result = Save::from_file(&path);
    assert!(
        result.is_err(),
        "truncated save should return Err, got {result:?}"
    );

    let _ = fs::remove_dir_all(&tmp);
}

// ===========================================================================
// 2. flipped_byte
// ===========================================================================

#[test]
fn flipped_byte_save() {
    let tmp = tmp_dir("flipped-save");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).expect("create tmp dir");

    let save = build_save();
    let path = tmp.join("save.sav");
    save.to_file(&path).expect("save to file");

    // Flip one byte in the trailing digest (last 32 bytes).
    // The body stays valid so postcard decodes fine; the digest check
    // then produces DigestMismatch.
    let mut data = fs::read(&path).expect("read save");
    let data_len = data.len();
    let flip_pos = data_len - 1; // last byte of the 32-byte digest
    data[flip_pos] ^= 0xFF;
    fs::write(&path, &data).expect("write corrupted data");

    // Save::from_file should return DigestMismatch
    let result = Save::from_file(&path);
    match result {
        Err(SaveError::DigestMismatch { .. }) => { /* expected */ }
        other => panic!("expected DigestMismatch, got {other:?}"),
    }

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn flipped_byte_pack() {
    let tmp = tmp_dir("flipped-pack");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).expect("create tmp dir");

    // Build a pack, save it, so we have content.pack + content.pack.blake3
    let pack = build_pack();
    pack.save(&tmp.join("content.pack")).expect("save pack");

    // Flip one byte in the middle of content.pack
    let pack_path = tmp.join("content.pack");
    let mut data = fs::read(&pack_path).expect("read pack");
    let mid = data.len() / 2;
    data[mid] ^= 0xFF;
    fs::write(&pack_path, &data).expect("write corrupted pack");

    // Pack::load_from_dir should return DigestMismatch
    let result = Pack::load_from_dir(&tmp);
    match result {
        Err(SaveError::DigestMismatch { .. }) => { /* expected */ }
        other => panic!("expected DigestMismatch, got {other:?}"),
    }

    let _ = fs::remove_dir_all(&tmp);
}

// ===========================================================================
// 3. future_schema_version
// ===========================================================================

#[test]
fn future_schema_version() {
    // Save::new() has assert!(version <= MAX_SCHEMA_VERSION), so we manually
    // construct bytes with schema_version = 9999.
    // Load order: after decoding the body it checks version > CURRENT_SCHEMA_VERSION
    // (which is 2) and returns Err(SaveError::Deserialize(...)) — before digest
    // verification, so the trailing digest can be garbage.

    let root = repo_root().join("content");
    let pack = Pack::from_content(&root).expect("pack should build");
    let content_digest = *blake3::hash(&pack.to_bytes()).as_bytes();

    let world = World::new(0);

    // Postcard-encode (9999u16, "0.1.0", content_digest, &world)
    let body = postcard::to_stdvec(&(9999u16, "0.1.0", content_digest, &world))
        .expect("encoding should succeed");

    // Append a garbage 32-byte digest (won't be checked — version check comes first)
    let mut data = body;
    data.extend_from_slice(&[0xABu8; 32]);

    // Load must reject the wildly-future version
    let result = Save::load(&data);
    assert!(
        result.is_err(),
        "future schema version should be rejected, got {result:?}"
    );
    match &result {
        Err(SaveError::Deserialize(msg)) => {
            assert!(
                msg.contains("9999"),
                "error message should mention version 9999: {msg}"
            );
        }
        other => panic!("expected Deserialize error mentioning 9999, got {other:?}"),
    }
}

// ===========================================================================
// 4. dangling_reference
// ===========================================================================
//
// The bake pipeline includes reference_resolve which checks that every
// spawn-table entry references an existing enemy ID.  We write a bestiary
// file whose loot references a non-existent item, *and* a spawn table that
// references a non-existent enemy.  The bake fails on the second (the actual
// dangling-reference check in the pipeline).  The loot reference is a
// different class of dangling reference (checked at the Pack level in
// content_integrity.rs).

#[test]
fn dangling_reference() {
    let tmp = tmp_dir("dangling-ref");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp.join("bestiary")).expect("create bestiary");
    fs::create_dir_all(&tmp.join("items")).expect("create items");
    fs::create_dir_all(&tmp.join("regions")).expect("create regions");
    fs::create_dir_all(&tmp.join("spawn_tables")).expect("create spawn tables");
    fs::create_dir_all(&tmp.join("scenes").join("act1")).expect("create scenes");

    // Copy flags.ron from real content (needed by schema_check)
    let flags_src = repo_root().join("content").join("flags.ron");
    fs::copy(&flags_src, &tmp.join("flags.ron")).expect("copy flags.ron");

    // ── Enemy referencing non-existent item "GHOST_SWORD" in loot ──
    let enemy_ron = r#"(
        id: "ENM_TEST_RAIDER",
        name_key: "enm.test_raider.name",
        family: BANDIT,
        region_affinity: ["R99_TEST"],
        gate: Always,
        stats: Stats(hp: 10, atk: 5, def: 2, spd: 3),
        resist: [],
        abilities: [],
        loot: [("GHOST_SWORD", 1)],
        xp: 10,
        tier: 1,
        sprite: "test_raider",
    )"#;
    fs::write(tmp.join("bestiary").join("enm_test_raider.ron"), enemy_ron)
        .expect("write enemy");

    // ── Minimal item (not "GHOST_SWORD") so loot reference is dangling ──
    let item_ron = r#"Item(
        id: "ITM_RUSTY_DAGGER",
        name_key: "item.rusty_dagger.name",
        description_key: "item.rusty_dagger.desc",
        item_type: Equipment,
        heal_hp: None,
        effect: None,
        usable_in: [],
        value: 5,
    )"#;
    fs::write(tmp.join("items").join("itm_rusty_dagger.ron"), item_ron)
        .expect("write item");

    // ── Minimal region ──
    let region_ron = r#"(
        id: "R99_TEST",
        name_key: "region.r99.name",
        description_key: "region.r99.desc",
        tier: 1,
        connections: [],
        locked: false,
        gate: Always,
    )"#;
    fs::write(tmp.join("regions").join("R99.ron"), region_ron).expect("write region");

    // ── Spawn table referencing a NON-EXISTENT enemy ID ──
    // This is the dangling reference that the bake pipeline *does* catch
    // (via validate::reference_resolve).
    let spawn_ron = r#"SpawnTable(
        region: "R99_TEST",
        chapter_stage: 1,
        pool: 1,
        entries: [
            (enemy: "ENM_DOES_NOT_EXIST", weight: 1, gate: Always),
        ],
    )"#;
    fs::write(
        tmp.join("spawn_tables").join("R99-test.ron"),
        spawn_ron,
    )
    .expect("write spawn table");

    // Run bake — should fail because reference_resolve catches the
    // dangling spawn-table → enemy reference
    let result = mc_data::bake::bake(&tmp);
    assert!(
        result.is_err(),
        "bake should fail when spawn table references non-existent enemy"
    );
    if let Err(errors) = &result {
        let error_strs: Vec<&str> = errors.iter().map(|e| e.message.as_str()).collect();
        let combined = error_strs.join(" ");
        assert!(
            combined.contains("ENM_DOES_NOT_EXIST") || combined.contains("does not exist"),
            "error should mention the dangling enemy ID, got: {combined}"
        );
    }

    let _ = fs::remove_dir_all(&tmp);
}

// ===========================================================================
// 5. supernatural_family
// ===========================================================================

#[test]
fn supernatural_family() {
    // Write a .ron file with family: GOD (outside the closed set of 10
    // Family variants: VERMIN, BEAST, SEA, MAN_AT_ARMS, CRIMINAL, PRISONER,
    // TROOP, BANDIT, HAZARD, BOSS).  Deserialisation should fail.
    let tmp = tmp_dir("supernatural");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).expect("create tmp dir");

    let bad_enemy = r#"(
        id: "ENM_DEMON_LORD",
        name_key: "enm.demon_lord.name",
        family: GOD,
        region_affinity: ["R99_HELL"],
        gate: Always,
        stats: Stats(hp: 999, atk: 99, def: 99, spd: 50),
        resist: [],
        abilities: [],
        loot: [],
        xp: 9999,
        tier: 5,
        sprite: "demon_lord",
    )"#;

    let result: Result<mc_data::schema::enemy::Enemy, _> = ron::from_str(bad_enemy);
    assert!(
        result.is_err(),
        "RON with unknown family `GOD` should fail to parse, got {result:?}"
    );

    let _ = fs::remove_dir_all(&tmp);
}

// ===========================================================================
// 6. read_only_directory
// ===========================================================================

#[test]
fn read_only_directory() {
    let save = build_save();

    // Write to a path whose parent directory doesn't exist.
    // On Linux this returns ENOENT, which maps to Io(kind=NotFound).
    let bad_path = PathBuf::from("/nonexistent_xyz_dir_forced_failures/save.sav");

    let result = save.to_file(&bad_path);
    assert!(
        matches!(&result, Err(SaveError::Io(_))),
        "writing to a non-existent directory should return Io error, got {result:?}"
    );
}
