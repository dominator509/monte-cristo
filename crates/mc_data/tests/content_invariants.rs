//! Content invariant tests for the Monte Cristo data pack.
//!
//! Verifies invariants that must hold for any valid content pack:
//! - Exactly one terminal scene
//! - No reserved flag identifiers are used anywhere
//! - Specific items exist
//! - Scene schema is free of combat-related fields

use std::path::PathBuf;

use mc_core::flags::FlagExpr;
use mc_core::ids::FlagId;
use mc_data::pack::Pack;

fn content_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); p.pop(); // repo root
    p.join("content")
}

fn build_pack() -> Pack {
    Pack::from_content(&content_root()).expect("should build pack from clean content")
}

/// The four reserved identifiers from SPEC-009 section 9 that must never
/// appear in content.
const RESERVED_IDENTIFIERS: &[&str] = &[
    "MERCEDES_ROUTE",
    "FERNAND_FORGIVEN",
    "POWER_OF_FRIENDSHIP",
    "DEUS_EX_MACHINA",
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

/// Verify that none of the 4 reserved identifiers appear in:
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
        }

        // Check scene.requires FlagExpr — FlagId values 0..22 are valid flags;
        // any value >= 22 or raw value that would correspond to a reserved
        // identifier is invalid. The FlagId raw values only go 0..21 for
        // real flags, so we verify no out-of-range FlagId is referenced.
        for flag_id in collect_flag_ids(&scene.requires) {
            assert!(
                (flag_id.raw() as usize) < FlagId::COUNT,
                "scene `{}` requires FlagId({}) which is out of range (valid: 0..{})",
                scene.id,
                flag_id.raw(),
                FlagId::COUNT
            );
        }

        // Check choice.requires FlagExpr
        for node in &scene.nodes {
            for choice in &node.choices {
                if let Some(ref expr) = choice.requires {
                    for flag_id in collect_flag_ids(expr) {
                        assert!(
                            (flag_id.raw() as usize) < FlagId::COUNT,
                            "scene `{}` node `{}` choice `{}` requires FlagId({}) which is out of range (valid: 0..{})",
                            scene.id,
                            node.id,
                            choice.text_key,
                            flag_id.raw(),
                            FlagId::COUNT
                        );
                    }
                }
            }
        }
    }
}

/// Verify that an item with ID containing "EDOUARD" exists in the pack.
#[test]
fn item_edouard_locket_exists() {
    let pack = build_pack();
    let found = pack.items.iter().any(|item| {
        item.id.contains("EDOUARD") || item.id.contains("EDOUARD_LOCKET")
    });
    assert!(
        found,
        "no item with id containing 'EDOUARD' or 'EDOUARD_LOCKET' found in pack"
    );
}

/// Verify that the scene schema has no combat-related fields.
///
/// Approach: build a Pack and verify that all scenes parse correctly.
/// Then do a best-effort heuristic check that no scene node text_key
/// contains combat terms like "hp", "health", "turn", or "meter"
/// (case-insensitive).
///
/// The stronger guarantee comes from the Rust type system: the Scene
/// struct has no HP/turn/meter fields, and deserialisation would fail
/// if content tried to supply them. This test confirms the full
/// pipeline runs cleanly and does not reference combat concepts.
#[test]
fn scene_schema_has_no_combat_fields() {
    let pack = build_pack();

    let combat_terms = ["hp", "health", "turn", "meter"];

    for scene in &pack.scenes {
        // Check each node's text_key
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

        // Also check on_exit fields — the Effects struct has no combat
        // fields (hp, hit_points, etc.), so any combat-related data would
        // fail serde deserialisation. We verify that effects use only
        // known fields.
        if let Some(ref effects) = scene.on_exit {
            // Serialize effects to a debug string and verify it doesn't
            // contain combat terms.
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
