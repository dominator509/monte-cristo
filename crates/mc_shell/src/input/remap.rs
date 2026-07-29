//! Input remapping: storage, round-trip, persistence.
//!
//! SPEC-004 section 7. The remap is stored in settings and survives restart.
//! No action requires more than two simultaneous inputs.

use crate::config::InputMap;

/// Validate an input map: no action may have more than 2 bindings.
pub fn validate_map(map: &InputMap) -> bool {
    for (_action, bindings) in map.iter() {
        if bindings.len() > 2 {
            return false;
        }
    }
    true
}

/// Check if a remap assigns the same key to two actions (ambiguous binding).
pub fn has_ambiguous_bindings(map: &InputMap) -> bool {
    let mut seen = std::collections::HashSet::new();
    for (_action, bindings) in map.iter() {
        for key in bindings {
            if !seen.insert(key.clone()) {
                return true;
            }
        }
    }
    false
}
