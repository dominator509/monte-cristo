//! Valentine survival test (SPEC-001 §14, SPEC-009 §7).
//!
//! Proves that the poison/tolerance system produces Valentine's canonical
//! survival: Noirtier administers PSN_BRUCINE at 0.5 dose across 18
//! authored days, raising her tolerance to 1.8, which exceeds Heloise's
//! lethal attempt. Simulated, not scripted.
//!
//! Also proves that skipping the tolerance-building regimen results in
//! death from the same dose.

use mc_core::fx::Fx;
use mc_core::ids::{CharId, PoisonId};
use mc_core::poison::PoisonState;

// ── Q16.16 raw constants ────────────────────────────────────────────────────
// Computed as (value * 65536) rounded to nearest integer.
const RAW_0_5: i32 = 32_768; // 0.5  (Noirtier's daily dose)
const RAW_4_0: i32 = 262_144; // 4.0  (Heloise's dose = lethal_dose)
const RAW_1_6: i32 = 104_858; // 1.6  (threshold for Heloise survival)
const RAW_0_10: i32 = 6_554; // 0.10 (tolerance_step for brucine)

/// The number of days in Noirtier's tolerance-building regimen.
const REGIMEN_DAYS: u64 = 18;

/// Ticks between each daily dose (1 day ≈ 240 ticks for brucine's onset).
const DAY_TICKS: u64 = 240;

#[test]
fn brucine_tolerance_exceeds_one_point_six() {
    let mut state = PoisonState::new();
    let valentine = CharId::CHR_VALENTINE;
    let brucine = PoisonId::PSN_BRUCINE;

    // Noirtier's regimen: 0.5 brucine each day for 18 days.
    for day in 0..REGIMEN_DAYS {
        let tick = day * DAY_TICKS;
        let lethal = state.administer(valentine, brucine, Fx::from_raw(RAW_0_5), tick);
        assert!(
            !lethal,
            "Noirtier's dose of 0.5 must never be lethal on day {}",
            day
        );
    }

    let tolerance = state.tolerance_for(valentine, brucine);
    // Expected: 18 × 0.10 = 1.8
    let expected = Fx::from_int(REGIMEN_DAYS as i32).saturating_mul(Fx::from_raw(RAW_0_10));
    assert_eq!(
        tolerance, expected,
        "After {} days of 0.5 brucine, tolerance should be exactly 1.8 (18 × 0.10)",
        REGIMEN_DAYS,
    );

    // Assert tolerance exceeds 1.6 (the threshold that guarantees survival
    // of Heloise's dose).
    assert!(
        tolerance > Fx::from_raw(RAW_1_6),
        "Tolerance {:?} must exceed 1.6 after {} days of regimen",
        tolerance,
        REGIMEN_DAYS,
    );
}

#[test]
fn tolerance_allows_surviving_lethal_dose() {
    let mut state = PoisonState::new();
    let valentine = CharId::CHR_VALENTINE;
    let brucine = PoisonId::PSN_BRUCINE;

    // Build tolerance via Noirtier's regimen.
    for day in 0..REGIMEN_DAYS {
        let tick = day * DAY_TICKS;
        state.administer(valentine, brucine, Fx::from_raw(RAW_0_5), tick);
    }

    // Now administer Heloise's dose — this is the lethal_dose (4.0), which
    // would kill a character with zero tolerance.
    let final_tick = REGIMEN_DAYS * DAY_TICKS;
    let lethal = state.administer(valentine, brucine, Fx::from_raw(RAW_4_0), final_tick);

    // With tolerance of ~1.8, the effective lethal threshold is 4.0 + 1.8 = 5.8,
    // so 4.0 is sub-lethal.
    assert!(
        !lethal,
        "Valentine must survive Heloise's dose of 4.0 with tolerance built from the regimen"
    );
}

#[test]
fn skipping_regimen_causes_death() {
    let mut state = PoisonState::new();
    let valentine = CharId::CHR_VALENTINE;
    let brucine = PoisonId::PSN_BRUCINE;

    // No regimen — Heloise's dose of 4.0 is administered directly to a
    // character with zero tolerance.
    let lethal = state.administer(valentine, brucine, Fx::from_raw(RAW_4_0), 0);

    // Without any tolerance, a dose of 4.0 meets the lethal_dose exactly,
    // which kills the character.
    assert!(
        lethal,
        "Without the tolerance-building regimen, Heloise's dose of 4.0 must kill Valentine"
    );
}
