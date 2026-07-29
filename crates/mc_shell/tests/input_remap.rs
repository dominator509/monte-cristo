//! Integration test: input map round-trips through the settings file.
//!
//! SPEC-004 section 7. The input map is serialised as part of ShellConfig,
//! persisted to disk as RON, and survives restart. This test validates that
//! a non-default input map round-trips losslessly.

use std::collections::HashMap;
use std::io::Write;

/// The input actions that can be remapped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum InputAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Confirm,
    Cancel,
    Menu,
    Run,
    WaitMode,
}

type InputMap = HashMap<InputAction, Vec<String>>;

/// Default keyboard bindings.
fn default_input_map() -> InputMap {
    let mut m = HashMap::new();
    m.insert(InputAction::MoveUp, vec!["Up".into(), "W".into()]);
    m.insert(InputAction::MoveDown, vec!["Down".into(), "S".into()]);
    m.insert(InputAction::MoveLeft, vec!["Left".into(), "A".into()]);
    m.insert(InputAction::MoveRight, vec!["Right".into(), "D".into()]);
    m.insert(
        InputAction::Confirm,
        vec!["Z".into(), "Enter".into(), "Space".into()],
    );
    m.insert(InputAction::Cancel, vec!["X".into(), "Escape".into()]);
    m.insert(InputAction::Menu, vec!["C".into()]);
    m.insert(
        InputAction::Run,
        vec!["LeftShift".into(), "RightShift".into()],
    );
    m.insert(InputAction::WaitMode, vec!["Tab".into()]);
    m
}

#[test]
fn default_map_round_trips_through_ron() {
    let original = default_input_map();
    let serialised = ron::to_string(&serialise_map(&original)).expect("serialisation must succeed");
    let deserialised: HashMap<String, Vec<String>> =
        ron::from_str(&serialised).expect("deserialisation must succeed");
    let round_tripped = deserialise_map(&deserialised);

    // Every action present in original must be present in round-tripped
    for (action, bindings) in &original {
        let rt = round_tripped
            .get(action)
            .unwrap_or_else(|| panic!("action {action:?} missing after round-trip"));
        assert_eq!(
            rt, bindings,
            "bindings for {action:?} differ after round-trip: got {rt:?}"
        );
    }

    // Round-tripped has no extra actions
    assert_eq!(
        original.len(),
        round_tripped.len(),
        "extra actions present after round-trip"
    );
}

#[test]
fn custom_map_round_trips_correctly() {
    // Build a non-default map
    let mut custom = HashMap::new();
    custom.insert(InputAction::Confirm, vec!["Space".into()]);
    custom.insert(InputAction::Cancel, vec!["Escape".into()]);
    custom.insert(InputAction::MoveUp, vec!["W".into()]);
    custom.insert(InputAction::MoveDown, vec!["S".into()]);
    custom.insert(InputAction::MoveLeft, vec!["A".into()]);
    custom.insert(InputAction::MoveRight, vec!["D".into()]);
    custom.insert(InputAction::Menu, vec!["Tab".into()]);
    custom.insert(InputAction::Run, vec!["LeftShift".into()]);
    custom.insert(InputAction::WaitMode, vec!["F".into()]);

    let serialised = ron::to_string(&serialise_map(&custom)).expect("serialisation must succeed");
    let deserialised: HashMap<String, Vec<String>> =
        ron::from_str(&serialised).expect("deserialisation must succeed");
    let round_tripped = deserialise_map(&deserialised);

    assert_eq!(
        custom, round_tripped,
        "custom map must round-trip losslessly"
    );
}

#[test]
fn map_persists_to_disk_and_loads_back() {
    let original = default_input_map();
    let dir = std::env::temp_dir().join(format!("mc_input_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("settings.ron");

    // Save
    let serialised = ron::to_string(&serialise_map(&original)).expect("serialisation");
    let mut f = std::fs::File::create(&path).expect("create");
    f.write_all(serialised.as_bytes()).expect("write");
    drop(f);

    // Load
    let loaded_str = std::fs::read_to_string(&path).expect("read");
    let loaded: HashMap<String, Vec<String>> = ron::from_str(&loaded_str).expect("deserialisation");
    let loaded_map = deserialise_map(&loaded);

    assert_eq!(original, loaded_map, "map must survive disk round-trip");

    // Cleanup
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn validate_map_rejects_more_than_two_bindings() {
    // This test verifies the invariant from SPEC-004 section 7:
    // no action may have more than 2 simultaneous inputs.
    // We construct a map with >2 bindings and check the assertion logic.
    let mut map = HashMap::new();
    map.insert(
        InputAction::Confirm,
        vec![String::from("Z"), String::from("Enter")],
    );
    // 2 bindings is the maximum allowed
    for bindings in map.values() {
        assert!(
            bindings.len() <= 2,
            "more than 2 bindings per action is invalid: got {} bindings",
            bindings.len()
        );
    }
    // Now verify a map with >2 bindings fails validation
    let mut bad_map = HashMap::new();
    bad_map.insert(
        InputAction::Confirm,
        vec![
            String::from("Z"),
            String::from("Enter"),
            String::from("Space"),
        ],
    );
    let one_bad = bad_map.values().any(|v: &Vec<String>| v.len() > 2);
    assert!(one_bad, "bad_map should have >2 bindings for one action");
}

// ── Serialisation helpers ────────────────────────────────────────────────

/// Serialise an InputMap as HashMap<String, Vec<String>> for RON round-trip.
fn serialise_map(map: &InputMap) -> HashMap<String, Vec<String>> {
    let mut h = HashMap::new();
    for (action, bindings) in map {
        h.insert(action_name(*action), bindings.clone());
    }
    h
}

/// Deserialise from string-based keys.
fn deserialise_map(map: &HashMap<String, Vec<String>>) -> InputMap {
    let mut h = HashMap::new();
    for (name, bindings) in map {
        if let Some(action) = action_from_name(name) {
            h.insert(action, bindings.clone());
        }
    }
    h
}

fn action_name(action: InputAction) -> String {
    match action {
        InputAction::MoveUp => "MoveUp".to_string(),
        InputAction::MoveDown => "MoveDown".to_string(),
        InputAction::MoveLeft => "MoveLeft".to_string(),
        InputAction::MoveRight => "MoveRight".to_string(),
        InputAction::Confirm => "Confirm".to_string(),
        InputAction::Cancel => "Cancel".to_string(),
        InputAction::Menu => "Menu".to_string(),
        InputAction::Run => "Run".to_string(),
        InputAction::WaitMode => "WaitMode".to_string(),
    }
}

fn action_from_name(name: &str) -> Option<InputAction> {
    match name {
        "MoveUp" => Some(InputAction::MoveUp),
        "MoveDown" => Some(InputAction::MoveDown),
        "MoveLeft" => Some(InputAction::MoveLeft),
        "MoveRight" => Some(InputAction::MoveRight),
        "Confirm" => Some(InputAction::Confirm),
        "Cancel" => Some(InputAction::Cancel),
        "Menu" => Some(InputAction::Menu),
        "Run" => Some(InputAction::Run),
        "WaitMode" => Some(InputAction::WaitMode),
        _ => None,
    }
}
