//! The three-phase final encounter (SPEC-001 §13, SPEC-010).
//!
//! FinalEncounter models the climactic confrontation with Fernand Mondego
//! (Morcerf). It has three phases, each driven entirely by flags and scene
//! effects — no HP, no turn order, no meters.
//!
//! Phase2 is **damage-immune**: no amount of narrative pressure can transition
//! out of Phase2. The only exit is `Command::NameYourself`, which requires
//! ALL of the following flags:
//!   - FLG_MORCERF_YANINA_DOSSIER
//!   - FLG_MORCERF_ALBERT_WITHDRAWN
//!   - FLG_MERCEDES_RECOGNITION

use crate::ids::FlagId;
use crate::world::World;
use serde::{Deserialize, Serialize};

/// The three phases of the final confrontation (SPEC-001 §13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterPhase {
    /// Opening exchange — narrative pressure can advance to Phase2.
    Phase1,
    /// Moral impasse — damage-immune. Only NameYourself can advance.
    Phase2,
    /// Final resolution — narrative pressure can resolve the encounter.
    Phase3,
    /// Confrontation complete.
    Resolved,
}

/// The final confrontation encounter.
///
/// No HP, no turn order, no meters — phase transitions are flag-driven.
/// The type itself makes it impossible to store numeric combat state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinalEncounter {
    /// The current phase of the encounter.
    pub phase: EncounterPhase,
}

/// Error returned when `Command::NameYourself` is attempted without the
/// requisite flags being set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameYourselfError {
    /// The flag IDs that are still missing.
    pub missing_flags: Vec<FlagId>,
}

/// The three flags that must all be set for NameYourself to succeed.
pub const NAME_YOURSELF_REQUIREMENTS: [FlagId; 3] = [
    FlagId::FLG_MORCERF_YANINA_DOSSIER,
    FlagId::FLG_MORCERF_ALBERT_WITHDRAWN,
    FlagId::FLG_MERCEDES_RECOGNITION,
];

impl FinalEncounter {
    /// Create a new final encounter starting in Phase1.
    pub fn new() -> Self {
        FinalEncounter {
            phase: EncounterPhase::Phase1,
        }
    }

    /// Attempt to advance from Phase1 to Phase2.
    ///
    /// Sets FLG_FINAL_PHASE1 if the transition occurs.
    /// Returns `true` if the phase changed.
    pub fn advance_from_phase1(&mut self, world: &mut World) -> bool {
        if self.phase != EncounterPhase::Phase1 {
            return false;
        }
        world.flags.set(FlagId::FLG_FINAL_PHASE1);
        self.phase = EncounterPhase::Phase2;
        true
    }

    /// Attempt the NameYourself command (Phase2 → Phase3 gate).
    ///
    /// Requires ALL of:
    ///   - FLG_MORCERF_YANINA_DOSSIER
    ///   - FLG_MORCERF_ALBERT_WITHDRAWN
    ///   - FLG_MERCEDES_RECOGNITION
    ///
    /// Returns `NameYourselfError` listing any missing flags.
    pub fn command_name_yourself(&mut self, world: &World) -> Result<(), NameYourselfError> {
        if self.phase != EncounterPhase::Phase2 {
            return Err(NameYourselfError {
                missing_flags: vec![],
            });
        }
        let missing: Vec<FlagId> = NAME_YOURSELF_REQUIREMENTS
            .iter()
            .copied()
            .filter(|f| !world.flags.is_set(*f))
            .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(NameYourselfError {
                missing_flags: missing,
            })
        }
    }

    /// Execute the NameYourself command (Phase2 → Phase3).
    ///
    /// After this succeeds, the encounter transitions to Phase3 and
    /// FLG_FINAL_PHASE2 is set.
    pub fn execute_name_yourself(&mut self, world: &mut World) {
        world.flags.set(FlagId::FLG_FINAL_PHASE2);
        self.phase = EncounterPhase::Phase3;
    }

    /// Attempt to resolve Phase3 (transition to Resolved).
    ///
    /// Sets FLG_FINAL_PHASE3. Returns `true` if the phase changed.
    pub fn resolve_phase3(&mut self, world: &mut World) -> bool {
        if self.phase != EncounterPhase::Phase3 {
            return false;
        }
        world.flags.set(FlagId::FLG_FINAL_PHASE3);
        self.phase = EncounterPhase::Resolved;
        true
    }

    /// Apply max-narrative-damage effects to the current phase.
    ///
    /// In Phase1: sets FLG_FINAL_PHASE1 and advances to Phase2.
    /// In Phase2: **damage-immune** — returns `false` and does nothing.
    /// In Phase3: sets FLG_FINAL_PHASE3 and advances to Resolved.
    /// In Resolved: no-op.
    ///
    /// This is the mechanism the confidence test uses to prove Phase2
    /// cannot be ended by any amount of applied pressure.
    ///
    /// Returns `true` if the phase changed.
    pub fn apply_damage(&mut self, world: &mut World) -> bool {
        match self.phase {
            EncounterPhase::Phase1 => self.advance_from_phase1(world),
            EncounterPhase::Phase2 => {
                // Phase2 is damage-immune — no amount of pressure
                // can transition out of Phase2.
                false
            }
            EncounterPhase::Phase3 => self.resolve_phase3(world),
            EncounterPhase::Resolved => false,
        }
    }
}

impl Default for FinalEncounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_phase1() {
        let fe = FinalEncounter::new();
        assert_eq!(fe.phase, EncounterPhase::Phase1);
    }

    #[test]
    fn advance_from_phase1_to_phase2() {
        let mut fe = FinalEncounter::new();
        let mut world = World::new(0);
        assert!(fe.advance_from_phase1(&mut world));
        assert_eq!(fe.phase, EncounterPhase::Phase2);
        assert!(world.flags.is_set(FlagId::FLG_FINAL_PHASE1));
    }

    #[test]
    fn advance_from_phase1_idempotent() {
        let mut fe = FinalEncounter::new();
        let mut world = World::new(0);
        assert!(fe.advance_from_phase1(&mut world));
        assert!(!fe.advance_from_phase1(&mut world));
    }

    #[test]
    fn name_yourself_rejected_wrong_phase() {
        let mut fe = FinalEncounter::new();
        let world = World::new(0);
        let result = fe.command_name_yourself(&world);
        assert!(result.is_err());
        // Wrong phase — empty missing list
        assert!(result.unwrap_err().missing_flags.is_empty());
    }

    #[test]
    fn name_yourself_succeeds_with_all_flags() {
        let mut fe = FinalEncounter::new();
        let mut world = World::new(0);
        fe.phase = EncounterPhase::Phase2;

        // Set all three required flags.
        world.flags.set(FlagId::FLG_MORCERF_YANINA_DOSSIER);
        world.flags.set(FlagId::FLG_MORCERF_ALBERT_WITHDRAWN);
        world.flags.set(FlagId::FLG_MERCEDES_RECOGNITION);

        let result = fe.command_name_yourself(&world);
        assert!(result.is_ok());
    }

    #[test]
    fn execute_name_yourself_transitions_to_phase3() {
        let mut fe = FinalEncounter::new();
        let mut world = World::new(0);
        fe.phase = EncounterPhase::Phase2;

        world.flags.set(FlagId::FLG_MORCERF_YANINA_DOSSIER);
        world.flags.set(FlagId::FLG_MORCERF_ALBERT_WITHDRAWN);
        world.flags.set(FlagId::FLG_MERCEDES_RECOGNITION);

        fe.execute_name_yourself(&mut world);
        assert_eq!(fe.phase, EncounterPhase::Phase3);
        assert!(world.flags.is_set(FlagId::FLG_FINAL_PHASE2));
    }

    #[test]
    fn resolve_phase3() {
        let mut fe = FinalEncounter::new();
        let mut world = World::new(0);
        fe.phase = EncounterPhase::Phase3;

        assert!(fe.resolve_phase3(&mut world));
        assert_eq!(fe.phase, EncounterPhase::Resolved);
        assert!(world.flags.is_set(FlagId::FLG_FINAL_PHASE3));
    }

    #[test]
    fn apply_damage_phase1_advances() {
        let mut fe = FinalEncounter::new();
        let mut world = World::new(0);
        assert!(fe.apply_damage(&mut world));
        assert_eq!(fe.phase, EncounterPhase::Phase2);
    }

    #[test]
    fn apply_damage_phase2_immune() {
        let mut fe = FinalEncounter::new();
        let mut world = World::new(0);
        fe.phase = EncounterPhase::Phase2;
        assert!(!fe.apply_damage(&mut world));
        assert_eq!(fe.phase, EncounterPhase::Phase2);
    }

    #[test]
    fn apply_damage_phase3_resolves() {
        let mut fe = FinalEncounter::new();
        let mut world = World::new(0);
        fe.phase = EncounterPhase::Phase3;
        assert!(fe.apply_damage(&mut world));
        assert_eq!(fe.phase, EncounterPhase::Resolved);
    }

    #[test]
    fn apply_damage_resolved_noop() {
        let mut fe = FinalEncounter::new();
        let mut world = World::new(0);
        fe.phase = EncounterPhase::Resolved;
        assert!(!fe.apply_damage(&mut world));
        assert_eq!(fe.phase, EncounterPhase::Resolved);
    }
}
