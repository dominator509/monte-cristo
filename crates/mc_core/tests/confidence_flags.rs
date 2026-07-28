//! Confidence test: Phase2 is damage-immune (SPEC-001 §13, SPEC-010).
//!
//! Proves that even after 10,000 ticks of maximum narrative damage applied
//! to Phase2 of the final encounter, Phase2 does not transition out.
//! This is a structural property of the type — no amount of pressure can
//! escape Phase2; only `Command::NameYourself` can.

use mc_core::final_encounter::{EncounterPhase, FinalEncounter};
use mc_core::world::World;

/// The number of damage ticks to apply.
const DAMAGE_TICKS: u64 = 10_000;

#[test]
fn phase2_damage_immune_after_ten_thousand_ticks() {
    let mut world = World::new(42);
    let mut encounter = FinalEncounter::new();

    // Advance to Phase2 naturally.
    assert!(encounter.advance_from_phase1(&mut world));
    assert_eq!(encounter.phase, EncounterPhase::Phase2);

    // Apply maximum narrative damage 10,000 times.
    for tick in 0..DAMAGE_TICKS {
        // At each tick we also max out mask and trust to prove that even
        // with maximum possible narrative pressure, Phase2 is immune.
        world.mask = i16::MAX;
        // Set trust to max for all characters
        world.trust.insert(
            mc_core::ids::CharId::CHR_EDMOND,
            i16::MAX,
        );

        let changed = encounter.apply_damage(&mut world);
        assert!(
            !changed,
            "Phase2 must never transition out via damage, but it did on tick {}",
            tick
        );
        assert_eq!(
            encounter.phase,
            EncounterPhase::Phase2,
            "Phase2 must remain Phase2 after tick {} of damage",
            tick
        );
    }

    // After 10,000 ticks, Phase2 must still be Phase2.
    assert_eq!(encounter.phase, EncounterPhase::Phase2);
    // The Phase2 flag must NOT be set (that's reserved for NameYourself).
    assert!(
        !world.flags.is_set(mc_core::ids::FlagId::FLG_FINAL_PHASE2),
        "FLG_FINAL_PHASE2 must not be set by damage in Phase2"
    );
}
