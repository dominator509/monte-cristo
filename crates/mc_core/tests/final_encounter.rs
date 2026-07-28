//! Final encounter NameYourself gate tests (SPEC-001 §13, SPEC-010).
//!
//! Proves that Command::NameYourself is rejected unless ALL of the following
//! three flags are set:
//!   - FLG_MORCERF_YANINA_DOSSIER
//!   - FLG_MORCERF_ALBERT_WITHDRAWN
//!   - FLG_MERCEDES_RECOGNITION
//!
//! Each test validates rejection when a single flag is absent while the other
//! two are present.

use mc_core::final_encounter::{EncounterPhase, FinalEncounter, NAME_YOURSELF_REQUIREMENTS};
use mc_core::ids::FlagId;
use mc_core::world::World;

/// Helper: create a FinalEncounter in Phase2 with two of the three required
/// flags set, and the absent flag named in `absent`.
fn encounter_missing_flag(absent: FlagId) -> (World, FinalEncounter) {
    let mut world = World::new(42);
    let mut encounter = FinalEncounter::new();

    // Advance to Phase2.
    encounter.advance_from_phase1(&mut world);
    assert_eq!(encounter.phase, EncounterPhase::Phase2);

    // Set the two flags that are NOT absent.
    let all_flags: Vec<FlagId> = NAME_YOURSELF_REQUIREMENTS.to_vec();
    for &f in &all_flags {
        if f != absent {
            world.flags.set(f);
        }
    }

    // Verify the absent flag is indeed missing.
    assert!(
        !world.flags.is_set(absent),
        "Flag {:?} should not be set for this test",
        absent
    );
    // Verify the other two are present.
    let others_present: bool = all_flags
        .iter()
        .filter(|&&f| f != absent)
        .all(|&f| world.flags.is_set(f));
    assert!(others_present, "Expected two flags to be set, one absent");

    (world, encounter)
}

#[test]
fn name_yourself_rejected_without_yanina_dossier() {
    let (world, mut encounter) = encounter_missing_flag(FlagId::FLG_MORCERF_YANINA_DOSSIER);

    let result = encounter.command_name_yourself(&world);
    assert!(
        result.is_err(),
        "NameYourself must be rejected when FLG_MORCERF_YANINA_DOSSIER is absent"
    );

    let err = result.unwrap_err();
    assert!(
        err.missing_flags
            .contains(&FlagId::FLG_MORCERF_YANINA_DOSSIER),
        "Error must list FLG_MORCERF_YANINA_DOSSIER as missing, got: {:?}",
        err.missing_flags
    );
    // Phase should remain Phase2.
    assert_eq!(encounter.phase, EncounterPhase::Phase2);
}

#[test]
fn name_yourself_rejected_without_albert_withdrawn() {
    let (world, mut encounter) = encounter_missing_flag(FlagId::FLG_MORCERF_ALBERT_WITHDRAWN);

    let result = encounter.command_name_yourself(&world);
    assert!(
        result.is_err(),
        "NameYourself must be rejected when FLG_MORCERF_ALBERT_WITHDRAWN is absent"
    );

    let err = result.unwrap_err();
    assert!(
        err.missing_flags
            .contains(&FlagId::FLG_MORCERF_ALBERT_WITHDRAWN),
        "Error must list FLG_MORCERF_ALBERT_WITHDRAWN as missing, got: {:?}",
        err.missing_flags
    );
    // Phase should remain Phase2.
    assert_eq!(encounter.phase, EncounterPhase::Phase2);
}

#[test]
fn name_yourself_rejected_without_mercedes_recognition() {
    let (world, mut encounter) = encounter_missing_flag(FlagId::FLG_MERCEDES_RECOGNITION);

    let result = encounter.command_name_yourself(&world);
    assert!(
        result.is_err(),
        "NameYourself must be rejected when FLG_MERCEDES_RECOGNITION is absent"
    );

    let err = result.unwrap_err();
    assert!(
        err.missing_flags
            .contains(&FlagId::FLG_MERCEDES_RECOGNITION),
        "Error must list FLG_MERCEDES_RECOGNITION as missing, got: {:?}",
        err.missing_flags
    );
    // Phase should remain Phase2.
    assert_eq!(encounter.phase, EncounterPhase::Phase2);
}

#[test]
fn name_yourself_succeeds_with_all_three_flags() {
    let mut world = World::new(42);
    let mut encounter = FinalEncounter::new();

    // Advance to Phase2.
    encounter.advance_from_phase1(&mut world);
    assert_eq!(encounter.phase, EncounterPhase::Phase2);

    // Set ALL three required flags.
    for &f in NAME_YOURSELF_REQUIREMENTS.iter() {
        world.flags.set(f);
    }

    // Check the gate.
    let result = encounter.command_name_yourself(&world);
    assert!(
        result.is_ok(),
        "NameYourself must succeed when all three flags are set"
    );

    // Execute the transition to Phase3.
    encounter.execute_name_yourself(&mut world);
    assert_eq!(
        encounter.phase,
        EncounterPhase::Phase3,
        "After NameYourself, encounter must enter Phase3"
    );
}
