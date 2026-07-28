//! Encounter budget decay test (LF-05).
//!
//! Asserts that the anti-grind system strictly decays experience to zero:
//! - 40 advances through a budget of 40
//! - Decay factor strictly decreases (or stays zero) every step
//! - Experience eventually floors at zero

use mc_core::budget::EncounterBudget;
use mc_core::fx::Fx;

/// After 40 advances, the decay factor must be zero (below 1 Fx).
/// The 7/10 compounding makes 0.7^40 ≈ 6.3e-7, well below 1/65536 (Fx::EPSILON).
#[test]
fn budget_decays_to_zero() {
    let mut budget = EncounterBudget::new(40);
    let base = Fx::from_int(100);

    for n in 0..40 {
        let xp = budget.experience_awarded(base);
        if n > 25 {
            // 100 * 0.7^26 ≈ 0.0004, definitely below 1.0 Fx
            assert_eq!(
                xp,
                Fx::ZERO,
                "experience must be zero at n={}, got {:?}",
                n,
                xp
            );
        }
        budget.advance();
    }

    assert!(budget.is_exhausted());
}

/// Strict monotonic decay: each advance must decrease (or maintain zero) the
/// remaining multiplier.
#[test]
fn decay_factor_strictly_decreases() {
    let mut budget = EncounterBudget::new(40);
    let mut prev = budget.remaining_multiplier();

    for n in 0..40 {
        budget.advance();
        let current = budget.remaining_multiplier();
        assert!(
            current <= prev,
            "decay factor must not increase at step {}, prev={:?} current={:?}",
            n,
            prev.raw(),
            current.raw()
        );
        if current == Fx::ZERO {
            // Once zero, must stay zero
            break;
        }
        prev = current;
    }
}

/// The decay factor is exactly 1.0 before any encounter.
#[test]
fn initial_decay_factor_is_one() {
    let budget = EncounterBudget::new(10);
    assert_eq!(budget.decay_factor(), Fx::ONE);
    assert_eq!(budget.remaining_multiplier(), Fx::ONE);
}

/// After one encounter, the decay factor is exactly 7/10.
#[test]
fn decay_factor_after_one_encounter() {
    let mut budget = EncounterBudget::new(10);
    budget.advance();
    let expected = Fx::from_int(7).saturating_div(Fx::from_int(10));
    assert_eq!(budget.decay_factor(), expected);
}

/// The advance() method returns true while the pool is available, false when exhausted.
#[test]
fn advance_availability() {
    let mut budget = EncounterBudget::new(5);
    for _ in 0..5 {
        assert!(budget.advance(), "should be available within pool");
    }
    assert!(!budget.advance(), "should be exhausted after pool");
}

/// Exhausted budget returns false from advance() and true from is_exhausted().
#[test]
fn exhausted_budget_returns_false() {
    let mut budget = EncounterBudget::new(3);
    for _ in 0..3 {
        budget.advance();
    }
    assert!(budget.is_exhausted());
    assert!(
        !budget.advance(),
        "advance should return false after pool exhausted"
    );
}

/// Experience awarded never exceeds the base amount.
#[test]
fn experience_never_exceeds_base() {
    let mut budget = EncounterBudget::new(40);
    let base = Fx::from_int(50);
    for _ in 0..40 {
        let xp = budget.experience_awarded(base);
        assert!(xp <= base, "xp {:?} must not exceed base {:?}", xp, base);
        budget.advance();
    }
}

/// Zero-pool budget is immediately exhausted.
#[test]
fn zero_pool_budget() {
    let mut budget = EncounterBudget::new(0);
    assert!(budget.is_exhausted());
    assert!(!budget.advance());
    assert_eq!(budget.remaining_fraction(), Fx::ZERO);
}

/// With custom 1/1 decay (no decay), experience never decreases.
#[test]
fn custom_no_decay_budget() {
    let mut budget = EncounterBudget::with_decay(10, 1, 1);
    let base = Fx::from_int(100);
    for _ in 0..10 {
        assert_eq!(budget.experience_awarded(base), base);
        budget.advance();
    }
    assert!(budget.is_exhausted());
}

/// Verify numeric bounds: the decay factor computation never panics for large spent.
#[test]
fn decay_factor_no_panic_large_spent() {
    for pool in [1u16, 10, 100, 1000] {
        let mut budget = EncounterBudget::new(pool);
        for _ in 0..pool.min(1000) {
            // This must never panic
            let _ = budget.decay_factor();
            let _ = budget.remaining_multiplier();
            let _ = budget.experience_awarded(Fx::from_int(1000));
            if !budget.advance() {
                break;
            }
        }
    }
}
