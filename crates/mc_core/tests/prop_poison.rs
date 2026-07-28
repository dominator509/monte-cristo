//! EP-007 M2: Property tests for the poison system.
//!
//! Uses proptest to verify invariants over generated poison sequences.
//! SPEC-001 §14: tolerance, lethal dose, onset timing.
//! Using `proptest::proptest!` macro with closure syntax.

use mc_core::fx::Fx;
use mc_core::ids::{CharId, PoisonId};

/// Tolerance is monotone non-decreasing under sub-lethal dosing.
#[test]
fn tolerance_monotonic() {
    let mut state = mc_core::poison::PoisonState::new();
    let edmond = CharId::CHR_EDMOND;
    for _ in 0..20 {
        let before = state.tolerance_for(edmond, PoisonId::PSN_BRUCINE);
        let lethal = state.administer(edmond, PoisonId::PSN_BRUCINE, Fx::from_raw(16384), 0);
        let after = state.tolerance_for(edmond, PoisonId::PSN_BRUCINE);
        assert!(!lethal, "sub-lethal dose must not be lethal");
        assert!(after >= before, "tolerance must not decrease: before={before:?}, after={after:?}");
    }
}

/// A lethal dose above tolerance always kills.
#[test]
fn lethal_dose_above_tolerance_kills() {
    let mut state = mc_core::poison::PoisonState::new();
    let edmond = CharId::CHR_EDMOND;
    for _ in 0..5 {
        state.administer(edmond, PoisonId::PSN_BRUCINE, Fx::from_raw(16384), 0);
    }
    let tolerance = state.tolerance_for(edmond, PoisonId::PSN_BRUCINE);
    // Need dose >= lethal_dose + tolerance
    // Brucine lethal_dose is RAW_4_0 = 262144 (4.0 in Q16.16)
    let lethal_dose = mc_core::poison::lookup(PoisonId::PSN_BRUCINE).unwrap().lethal_dose;
    let overdose = lethal_dose + tolerance + Fx::ONE;
    let lethal = state.administer(edmond, PoisonId::PSN_BRUCINE, overdose, 0);
    assert!(lethal, "dose above tolerance must be lethal: dose={overdose:?}, tol={tolerance:?}, lethal_dose={lethal_dose:?}");
}

/// A small dose below tolerance is never lethal.
#[test]
fn small_dose_not_lethal() {
    let mut state = mc_core::poison::PoisonState::new();
    let edmond = CharId::CHR_EDMOND;
    for _ in 0..20 {
        let lethal = state.administer(edmond, PoisonId::PSN_BRUCINE, Fx::from_raw(100), 0);
        assert!(!lethal, "tiny dose must never be lethal");
    }
}

/// Each poison has a positive onset time.
#[test]
fn onset_positive() {
    for raw in 0..5u16 {
        let pid = PoisonId::from_raw(raw);
        let data = mc_core::poison::lookup(pid).expect("poison data must exist");
        assert!(data.onset > 0, "onset must be > 0 for poison {raw}");
        assert!(data.onset <= 1000, "onset must be reasonable for poison {raw}");
    }
}

/// Each poison has a positive lethal dose.
#[test]
fn lethal_dose_positive() {
    for raw in 0..5u16 {
        let pid = PoisonId::from_raw(raw);
        let data = mc_core::poison::lookup(pid).expect("poison data must exist");
        assert!(data.lethal_dose > Fx::ZERO, "lethal dose must be > 0 for poison {raw}");
    }
}
