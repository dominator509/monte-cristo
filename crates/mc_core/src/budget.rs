//! Encounter budget — anti-grind system (INV-12).
//!
//! Per `(RegionId, ChapterId)`, tracks how many encounters have been cleared
//! in that region-chapter. Experience awarded decays by 7/10 compounding per
//! cleared encounter, flooring at zero. Once the pool is spent, the region
//! stops spawning.

use crate::fx::Fx;
use serde::{Deserialize, Serialize};

/// Anti-grind encounter budget (SPEC-001 section 11, INV-12).
///
/// # Fields
///
/// * `pool` — total encounters authored for this region-chapter.
/// * `spent` — encounters already cleared this region-chapter.
/// * `decay_num` — numerator of the decay factor (default 7).
/// * `decay_den` — denominator of the decay factor (default 10).
///
/// Experience for the `n`-th cleared encounter is `base × (decay_num/decay_den)^n`,
/// floored to zero once below 1 `Fx`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterBudget {
    /// Total authored encounters for this region-chapter pair.
    pub pool: u16,
    /// Encounters already cleared.
    pub spent: u16,
    /// Decay numerator (default 7).
    pub decay_num: u16,
    /// Decay denominator (default 10).
    pub decay_den: u16,
}

impl EncounterBudget {
    /// Create a new budget with the default 7/10 decay ratio.
    pub fn new(pool: u16) -> Self {
        EncounterBudget {
            pool,
            spent: 0,
            decay_num: 7,
            decay_den: 10,
        }
    }

    /// Create a budget with a custom decay ratio.
    pub fn with_decay(pool: u16, decay_num: u16, decay_den: u16) -> Self {
        assert!(decay_den != 0, "EncounterBudget: decay_den cannot be zero");
        EncounterBudget {
            pool,
            spent: 0,
            decay_num,
            decay_den,
        }
    }

    /// The decay multiplier as an `Fx` value: `(decay_num/decay_den)^spent`.
    ///
    /// This is computed iteratively — O(spent) — which is acceptable since
    /// `spent` is bounded by authored pool sizes (typically ≤ 40).
    pub fn decay_factor(&self) -> Fx {
        let num = Fx::from_int(self.decay_num as i32);
        let den = Fx::from_int(self.decay_den as i32);
        let ratio = num.saturating_div(den);
        let mut factor = Fx::ONE;
        for _ in 0..self.spent {
            factor = factor.saturating_mul(ratio);
        }
        factor
    }

    /// Remaining experience multiplier after `spent` encounters.
    ///
    /// Returns `Fx::ZERO` when the decay factor drops below 1 Fx.
    pub fn remaining_multiplier(&self) -> Fx {
        let mult = self.decay_factor();
        if mult < Fx::ONE {
            Fx::ZERO
        } else {
            mult
        }
    }

    /// Compute the actual experience awarded given the base XP for this enemy.
    ///
    /// Result is `base × (decay_num/decay_den)^spent`, floored to zero if
    /// the result is below 1 Fx.
    pub fn experience_awarded(&self, base: Fx) -> Fx {
        let mult = self.decay_factor();
        let decayed = base.saturating_mul(mult);
        if decayed < Fx::ONE {
            Fx::ZERO
        } else {
            decayed
        }
    }

    /// Mark one encounter as cleared.
    ///
    /// Returns `true` if the budget is still available (spent < pool).
    /// Returns `false` if the budget was already exhausted.
    pub fn advance(&mut self) -> bool {
        if self.spent < self.pool {
            self.spent += 1;
            true
        } else {
            false
        }
    }

    /// Whether the encounter pool is exhausted.
    ///
    /// Once exhausted, no more enemies should spawn for this region-chapter.
    pub fn is_exhausted(&self) -> bool {
        self.spent >= self.pool
    }

    /// Fraction of the budget remaining, as a ratio in `[0, 1]`.
    pub fn remaining_fraction(&self) -> Fx {
        if self.pool == 0 {
            return Fx::ZERO;
        }
        let remaining = self.pool.saturating_sub(self.spent);
        Fx::from_int(remaining as i32).saturating_div(Fx::from_int(self.pool as i32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_budget() {
        let b = EncounterBudget::new(10);
        assert_eq!(b.pool, 10);
        assert_eq!(b.spent, 0);
        assert_eq!(b.decay_num, 7);
        assert_eq!(b.decay_den, 10);
    }

    #[test]
    fn decay_factor_initial_is_one() {
        let b = EncounterBudget::new(10);
        assert_eq!(b.decay_factor(), Fx::ONE);
    }

    #[test]
    fn decay_factor_after_one() {
        let mut b = EncounterBudget::new(10);
        b.advance();
        // 7/10 = 0.7 in Fx
        let expected = Fx::from_int(7).saturating_div(Fx::from_int(10));
        assert_eq!(b.decay_factor(), expected);
    }

    #[test]
    fn decay_factor_after_two() {
        let mut b = EncounterBudget::new(10);
        b.advance();
        b.advance();
        // (7/10)^2 = 49/100 = 0.49
        let expected = Fx::from_int(7)
            .saturating_div(Fx::from_int(10))
            .saturating_mul(Fx::from_int(7).saturating_div(Fx::from_int(10)));
        assert_eq!(b.decay_factor(), expected);
    }

    #[test]
    fn advance_increments_spent() {
        let mut b = EncounterBudget::new(5);
        assert_eq!(b.spent, 0);
        assert!(b.advance());
        assert_eq!(b.spent, 1);
    }

    #[test]
    fn advance_exhausted_returns_false() {
        let mut b = EncounterBudget::new(2);
        assert!(b.advance());
        assert!(b.advance());
        assert!(!b.advance()); // pool exhausted
        assert_eq!(b.spent, 2);
    }

    #[test]
    fn is_exhausted_true_when_spent_reaches_pool() {
        let mut b = EncounterBudget::new(3);
        assert!(!b.is_exhausted());
        b.advance();
        b.advance();
        b.advance();
        assert!(b.is_exhausted());
    }

    #[test]
    fn experience_decays_to_zero() {
        let mut b = EncounterBudget::new(40);
        let base = Fx::from_int(100);
        for n in 0..40 {
            let xp = b.experience_awarded(base);
            if n > 25 {
                // By ~26 encounters, 100 * 0.7^26 ≈ 0.0004 < 1.0 Fx
                assert_eq!(xp, Fx::ZERO, "xp should be zero at n={}", n);
            }
            b.advance();
        }
    }

    #[test]
    fn remaining_fraction_full() {
        let b = EncounterBudget::new(10);
        assert_eq!(b.remaining_fraction(), Fx::ONE);
    }

    #[test]
    fn remaining_fraction_half() {
        let mut b = EncounterBudget::new(10);
        b.advance();
        b.advance();
        b.advance();
        b.advance();
        b.advance();
        assert_eq!(b.remaining_fraction(), Fx::HALF);
    }

    #[test]
    fn remaining_fraction_exhausted() {
        let mut b = EncounterBudget::new(5);
        for _ in 0..5 {
            b.advance();
        }
        assert_eq!(b.remaining_fraction(), Fx::ZERO);
    }

    #[test]
    fn experience_awarded_preserves_zero_base() {
        let mut b = EncounterBudget::new(10);
        assert_eq!(b.experience_awarded(Fx::ZERO), Fx::ZERO);
        b.advance();
        assert_eq!(b.experience_awarded(Fx::ZERO), Fx::ZERO);
    }

    #[test]
    fn custom_decay_ratio() {
        // Using 1/2 decay: each encounter halves XP
        let mut b = EncounterBudget::with_decay(10, 1, 2);
        assert_eq!(b.decay_factor(), Fx::ONE);
        b.advance();
        assert_eq!(b.decay_factor(), Fx::HALF);
        b.advance();
        assert_eq!(
            b.decay_factor(),
            Fx::from_int(1).saturating_div(Fx::from_int(4))
        );
    }
}
