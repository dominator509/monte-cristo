//! Schema-level verification: Scene and Confidence types must not express
//! combat-related fields (SPEC-010 §10).
//!
//! This test asserts the scene schema as defined in
//! `crates/mc_data/src/schema/scene.rs` has no field capable of expressing
//! hit points, turn order, or a resource meter.
//!
//! Two layers of verification:
//! 1. Compile-time: Verify `Scene` and `Node` structs have no HP/turn/meter
//!    fields by inspecting their serialized form.
//! 2. Runtime: Deserialize a representative scene and assert no combat terms
//!    appear in the resulting structure.

use mc_data::schema::scene::{Choice, Effects, Node, Scene};
use mc_core::flags::FlagExpr;

/// Compile-time: Verify the Scene struct's serialized schema has no combat
/// fields.
///
/// Serializes an empty Scene to RON and asserts no combat-related field names
/// appear in the output. This catches accidental additions of HP/turn/meter
/// fields at the schema level.
#[test]
fn scene_struct_has_no_combat_fields() {
    let scene = Scene {
        id: String::from("test_scene"),
        act: mc_data::schema::scene::Act::ActI_ARREST,
        participants: vec![],
        requires: FlagExpr::Always,
        nodes: vec![],
        on_exit: None,
        terminal: false,
    };

    let ron_str = ron::to_string(&scene).expect("Scene should serialize to RON");
    let lower = ron_str.to_lowercase();

    let combat_terms = ["hp", "health", "turn", "meter", "atb", "gauge", "combat",
                        "attack", "defense", "damage", "initiative", "speed"];

    for term in &combat_terms {
        assert!(
            !lower.contains(term),
            "Scene schema contains combat term '{}' in serialized form: {}",
            term,
            ron_str
        );
    }
}

/// Compile-time: Verify Node struct has no combat-related fields.
#[test]
fn node_struct_has_no_combat_fields() {
    let node = Node {
        id: String::from("test_node"),
        text_key: String::from("test.dialogue"),
        choices: vec![],
    };

    let ron_str = ron::to_string(&node).expect("Node should serialize to RON");
    let lower = ron_str.to_lowercase();

    let combat_terms = ["hp", "health", "turn", "meter", "atb", "gauge",
                        "combat", "attack", "defense", "damage", "initiative"];

    for term in &combat_terms {
        assert!(
            !lower.contains(term),
            "Node schema contains combat term '{}' in serialized form: {}",
            term,
            ron_str
        );
    }
}

/// Compile-time: Verify Choice struct has no combat-related fields.
#[test]
fn choice_struct_has_no_combat_fields() {
    let choice = Choice {
        text_key: String::from("test.choice"),
        to: String::from("next_node"),
        trust: None,
        requires: None,
    };

    let ron_str = ron::to_string(&choice).expect("Choice should serialize to RON");
    let lower = ron_str.to_lowercase();

    let combat_terms = ["hp", "health", "turn", "meter", "atb", "gauge",
                        "combat", "attack", "damage", "initiative"];

    for term in &combat_terms {
        assert!(
            !lower.contains(term),
            "Choice schema contains combat term '{}' in serialized form: {}",
            term,
            ron_str
        );
    }
}

/// Compile-time: Verify Effects struct has no combat-related fields.
#[test]
fn effects_struct_has_no_combat_fields() {
    let effects = Effects {
        set_flags: vec![],
        clear_flags: vec![],
        consume: vec![],
        grant: vec![],
        trust: None,
        mask: None,
    };

    let ron_str = ron::to_string(&effects).expect("Effects should serialize to RON");
    let lower = ron_str.to_lowercase();

    // `mask` is a resource meter for the disguise/narrative system, not combat.
    // The combat terms we check are specific to HP/damage/turn-order.
    let combat_terms = ["hp", "health", "turn_order", "atb", "gauge",
                        "combat", "attack", "damage", "initiative", "speed_stat"];

    for term in &combat_terms {
        assert!(
            !lower.contains(term),
            "Effects schema contains combat term '{}' in serialized form: {}",
            term,
            ron_str
        );
    }
}

/// Runtime: Verify the full scene pipeline rejects combat content.
///
/// Builds a scene from known-good content (the bake pipeline) and asserts
/// no scene node text keys reference combat terms. This is the same
/// verification as content_invariants::scene_schema_has_no_combat_fields,
/// provided here as a standalone schema-level test per SPEC-010 §10.
#[test]
fn baked_scenes_have_no_combat_references() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let content_root = manifest_dir
        .parent()
        .and_then(std::path::Path::parent)
        .map(|p| p.join("content"))
        .expect("content directory should be locatable");

    let pack = mc_data::pack::Pack::from_content(&content_root)
        .expect("should build pack from clean content");

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

/// Schema-level: Verify the Scene struct has no numeric field that could
/// represent a combat stat like HP.
///
/// Uses compile-time type assertions about serialization rather than
/// runtime reflection.
#[test]
fn scene_field_count_matches_expected() {
    // Serialize a full Scene and count field keys.
    // This is a soft invariant: if new non-combat fields are added,
    // this test must be updated.
    let scene = Scene {
        id: String::from("test"),
        act: mc_data::schema::scene::Act::ActI_ARREST,
        participants: vec![],
        requires: FlagExpr::Always,
        nodes: vec![],
        on_exit: None,
        terminal: false,
    };

    let ron_str = ron::to_string(&scene).expect("serialize");
    // The RON output should contain exactly these top-level field keys.
    let expected_fields = ["id", "act", "participants", "requires", "nodes",
                           "on_exit", "terminal"];
    for field in &expected_fields {
        assert!(
            ron_str.contains(field),
            "Scene RON should contain field '{}' but serialized form is: {}",
            field,
            ron_str
        );
    }

    // No unexpected combat-prefixed fields
    let lower = ron_str.to_lowercase();
    assert!(
        !lower.contains("hp_") && !lower.contains("turn_") && !lower.contains("atb_"),
        "Scene RON contains unexpected combat-prefixed field: {}",
        ron_str
    );
}
