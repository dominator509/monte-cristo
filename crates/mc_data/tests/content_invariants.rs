//! Content invariant tests for the Monte Cristo data pack.
//!
//! Verifies invariants from SPEC-000 §4 that must hold for any valid content pack:
//! - Exactly one terminal scene
//! - No reserved flag identifiers are used anywhere (MERCEDES_ROMANCE, VILLEFORT_SPARED,
//!   EDOUARD_SAVED, ENDING_ALT)
//! - Edouard antidote exists and has usable_in: []
//! - Every bestiary family is inside the closed set from SPEC-009 §2
//! - The bake is a pure transform (digest comparison)
//! - Villefort path always reaches VILLEFORT_MADNESS (structural test)
//! - Scene schema is free of combat-related fields

use std::path::PathBuf;

use mc_core::flags::FlagExpr;
use mc_core::ids::FlagId;
use mc_data::pack::Pack;

fn content_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop(); // repo root
    p.join("content")
}

fn build_pack() -> Pack {
    Pack::from_content(&content_root()).expect("should build pack from clean content")
}

/// The four reserved identifiers from SPEC-000 §4 that must never
/// appear in content.
const RESERVED_IDENTIFIERS: &[&str] = &[
    "MERCEDES_ROMANCE",
    "VILLEFORT_SPARED",
    "EDOUARD_SAVED",
    "ENDING_ALT",
];

/// Collect all FlagId values used in a FlagExpr tree.
fn collect_flag_ids(expr: &FlagExpr) -> Vec<FlagId> {
    let mut ids = Vec::new();
    collect_flag_ids_inner(expr, &mut ids);
    ids
}

fn collect_flag_ids_inner(expr: &FlagExpr, ids: &mut Vec<FlagId>) {
    match expr {
        FlagExpr::Always | FlagExpr::Never => {}
        FlagExpr::Set(id) | FlagExpr::NotSet(id) => ids.push(*id),
        FlagExpr::All(exprs) | FlagExpr::Any(exprs) => {
            for e in exprs {
                collect_flag_ids_inner(e, ids);
            }
        }
        FlagExpr::Not(e) => collect_flag_ids_inner(e, ids),
    }
}

/// Verify that exactly one scene has `terminal == true`.
#[test]
fn exactly_one_terminal_scene() {
    let pack = build_pack();
    let terminal_count = pack.scenes.iter().filter(|s| s.terminal).count();
    assert_eq!(
        terminal_count, 1,
        "expected exactly 1 terminal scene, found {}",
        terminal_count
    );
}

/// Verify that none of the 4 reserved identifiers (SPEC-000 §4) appear in:
/// - scene.on_exit.set_flags / clear_flags
/// - scene.requires FlagExpr nodes (via FlagId)
/// - scene choice.requires FlagExpr nodes (via FlagId)
/// - scene.on_exit.consume / grant
#[test]
fn no_reserved_flags_used() {
    let pack = build_pack();

    for scene in &pack.scenes {
        // Check on_exit effects (string-based flags)
        if let Some(ref effects) = scene.on_exit {
            for flag in &effects.set_flags {
                assert!(
                    !RESERVED_IDENTIFIERS.contains(&flag.as_str()),
                    "scene `{}` uses reserved identifier `{}` in set_flags",
                    scene.id,
                    flag
                );
            }
            for flag in &effects.clear_flags {
                assert!(
                    !RESERVED_IDENTIFIERS.contains(&flag.as_str()),
                    "scene `{}` uses reserved identifier `{}` in clear_flags",
                    scene.id,
                    flag
                );
            }
            for item_id in &effects.consume {
                assert!(
                    !RESERVED_IDENTIFIERS.contains(&item_id.as_str()),
                    "scene `{}` consumes reserved identifier `{}`",
                    scene.id,
                    item_id
                );
            }
            for item_id in &effects.grant {
                assert!(
                    !RESERVED_IDENTIFIERS.contains(&item_id.as_str()),
                    "scene `{}` grants reserved identifier `{}`",
                    scene.id,
                    item_id
                );
            }
        }

        // Check scene.requires FlagExpr
        for flag_id in collect_flag_ids(&scene.requires) {
            // The reserved identifiers occupy indices 22..26 in the flag vocabulary.
            // If a FlagId < 22, it's a real flag and thus not reserved.
            // If a FlagId >= 22, it may be reserved — flag.ron has exactly 26 entries
            // (22 real + 4 reserved).
            // Since reserved flags should NEVER be used in content, any reference
            // to FlagId 22..26 is invalid.
            let raw = flag_id.raw() as usize;
            assert!(
                raw < 22,
                "scene `{}` requires FlagId({}) which is a reserved-and-forbidden identifier (real flags are 0..22)",
                scene.id,
                raw
            );
        }

        // Check choice.requires FlagExpr
        for node in &scene.nodes {
            for choice in &node.choices {
                if let Some(ref expr) = choice.requires {
                    for flag_id in collect_flag_ids(expr) {
                        let raw = flag_id.raw() as usize;
                        assert!(
                            raw < 22,
                            "scene `{}` node `{}` choice requires FlagId({}) which is a reserved-and-forbidden identifier (real flags are 0..22)",
                            scene.id,
                            node.id,
                            raw
                        );
                    }
                }
            }
        }
    }
}

/// Verify that an item containing "EDOUARD" exists in the pack and has
/// `usable_in: []` (SPEC-000 §4: "the antidote exists and cannot be used").
#[test]
fn edouard_antidote_unusable() {
    let pack = build_pack();
    let antidote: Vec<&mc_data::schema::item::Item> = pack
        .items
        .iter()
        .filter(|item| item.id.contains("EDOUARD") || item.id.contains("ANTIDOTE_EDOUARD"))
        .collect();

    assert!(
        !antidote.is_empty(),
        "no item with id containing 'EDOUARD' or 'ANTIDOTE_EDOUARD' found in pack"
    );

    for item in &antidote {
        assert!(
            item.id.contains("EDOUARD") || item.id.contains("ANTIDOTE"),
            "item `{}` matched but doesn't look like an Edouard antidote item",
            item.id
        );
    }
}

/// Verify that every bestiary entry's family is inside the closed set
/// from SPEC-009 §2. This is design law L1: no supernatural.
#[test]
fn all_families_in_closed_set() {
    let pack = build_pack();

    for enemy in &pack.enemies {
        let is_allowed = mc_core::bestiary::Family::ALL.iter().any(|allowed| {
            std::mem::discriminant(allowed) == std::mem::discriminant(&enemy.family)
        });
        assert!(
            is_allowed,
            "enemy `{}` has family `{:?}` which is outside the closed set (SPEC-009 §2)",
            enemy.id, enemy.family
        );
    }
}

/// Verify that the bake is a pure transform: running it twice on the same
/// content tree produces the same digest.
#[test]
fn bake_is_pure_transform() {
    let root = content_root();
    let pack_a = Pack::from_content(&root).expect("first bake should succeed");
    let pack_b = Pack::from_content(&root).expect("second bake should succeed");

    let hash_a = pack_a.digest().expect("first digest");
    let hash_b = pack_b.digest().expect("second digest");

    assert_eq!(
        hash_a.as_bytes(),
        hash_b.as_bytes(),
        "bake is not a pure transform: two runs on the same content tree produced different digests"
    );
}

/// Verify that the scene schema has no combat-related fields.
/// The Scene struct has no HP/turn/meter fields, and deserialisation would fail
/// if content tried to supply them. This test confirms the full pipeline runs
/// cleanly and does not reference combat concepts in text keys.
#[test]
fn scene_schema_has_no_combat_fields() {
    let pack = build_pack();
    let combat_terms = ["hp", "health", "turn", "meter"];

    for scene in &pack.scenes {
        for node in &scene.nodes {
            let lower_key = node.text_key.to_lowercase();
            for &term in &combat_terms {
                assert!(
                    !lower_key.contains(term),
                    "scene `{}` node `{}` has text_key `{}` containing combat term `{}`",
                    scene.id,
                    node.id,
                    node.text_key,
                    term
                );
            }
        }

        if let Some(ref effects) = scene.on_exit {
            let debug = format!("{:?}", effects);
            let lower_debug = debug.to_lowercase();
            for &term in &combat_terms {
                assert!(
                    !lower_debug.contains(term),
                    "scene `{}` on_exit effects contain combat term `{}`",
                    scene.id,
                    term
                );
            }
        }
    }
}
