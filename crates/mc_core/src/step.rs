//! Step order — the single declared system dispatch order.
//!
//! Adding a system means adding it here and adding it to the determinism
//! property test. No dynamic registration.

/// The declared system order (SPEC-001 section 4).
pub const ORDER: &[&str] = &[
    "scene_advance",
    "calendar_advance",
    "season_advance",
    "field_movement",
    "spawn_resolution",
    "encounter_contact",
    "battle_atb",
    "battle_action_resolve",
    "status_tick",
    "poison_tick",
    "budget_decay",
    "flag_reactions",
    "event_flush",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_has_13_systems() {
        assert_eq!(ORDER.len(), 13);
    }

    #[test]
    fn order_is_unchanged() {
        assert_eq!(ORDER[0], "scene_advance");
        assert_eq!(ORDER[12], "event_flush");
    }
}
