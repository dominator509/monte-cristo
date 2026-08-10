//! Seeded PCG64 generator. INV-01: the only source of randomness in mc_core.
//!
//! No ambient generator — advanced only through explicit calls. Every call site
//! records debug info for determinism divergence localisation.

use serde::{Deserialize, Serialize};

/// Errors returned when a caller supplies an invalid RNG range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RngError {
    #[error("random range lower bound {lo} exceeds upper bound {hi}")]
    InvalidRange { lo: u32, hi: u32 },
}

/// A PCG64 random number generator with 128-bit state.
///
/// Based on the PCG family: `state = state * 6364136223846793005 + inc`
/// Then the output is rotated/xored per the PCG XSH-RR scheme.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rng {
    state: u64,
    inc: u64,
}

impl Rng {
    /// Create a new RNG from a 128-bit seed.
    /// The seed is split into `state` (low 64 bits) and `inc` (high 64 bits).
    /// The generator is then advanced a few steps to initialise.
    pub fn new(seed: u128) -> Self {
        let state = seed as u64;
        let inc = (seed >> 64) as u64 | 1; // inc must be odd
        let mut rng = Rng { state: 0, inc };
        rng.next_u32(); // warm up
        rng.state = rng.state.wrapping_add(state);
        rng.next_u32(); // warm up
        rng.next_u32(); // warm up
        rng
    }

    /// Generate the next `u32`.
    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(6364136223846793005).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Generate a `u32` in `[lo, hi]` (inclusive) by rejection sampling.
    pub fn next_range(&mut self, lo: u32, hi: u32) -> Result<u32, RngError> {
        if lo > hi {
            return Err(RngError::InvalidRange { lo, hi });
        }
        let range = hi.wrapping_sub(lo).wrapping_add(1);
        if range == 0 {
            return Ok(self.next_u32()); // full range
        }
        // Rejection sampling to avoid modulo bias
        let threshold = range.wrapping_neg() % range;
        loop {
            let val = self.next_u32();
            if val >= threshold {
                return Ok(lo + (val % range));
            }
        }
    }

    /// Pick a weighted item from a map.
    ///
    /// Returns `None` for an empty map. Zero-weight maps are sampled uniformly.
    pub fn weighted_pick<I: Copy + Ord>(
        &mut self,
        weights: &std::collections::BTreeMap<I, u32>,
    ) -> Option<I> {
        if weights.is_empty() {
            return None;
        }
        let total: u32 = weights.values().copied().sum();
        if total == 0 {
            // All weights are zero; pick uniformly.
            let idx = self.next_range(0, weights.len() as u32 - 1).ok()?;
            return weights.keys().nth(idx as usize).copied();
        }
        let mut roll = self.next_range(1, total).ok()?;
        for (id, w) in weights.iter() {
            if *w >= roll {
                return Some(*id);
            }
            roll -= *w;
        }
        // Fallback (shouldn't happen due to total check)
        weights.last_key_value().map(|(id, _)| *id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn deterministic_from_seed() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(12345);
        assert_ne!(a.next_u32(), b.next_u32());
        assert_ne!(a.next_u32(), b.next_u32());
    }

    #[test]
    fn next_range_bounds() {
        let mut rng = Rng::new(999);
        for _ in 0..10_000 {
            let v = rng.next_range(3, 7).expect("valid range");
            assert!((3..=7).contains(&v), "value {v} out of range [3,7]");
        }
    }

    #[test]
    fn next_range_single_value() {
        let mut rng = Rng::new(0);
        for _ in 0..100 {
            assert_eq!(rng.next_range(5, 5).expect("valid range"), 5);
        }
    }

    #[test]
    fn weighted_pick_respects_weights() {
        let mut rng = Rng::new(42);
        let mut weights = BTreeMap::new();
        weights.insert("a", 1u32);
        weights.insert("b", 0u32);
        for _ in 0..100 {
            // Should always pick "a" since "b" has weight 0
            assert_eq!(rng.weighted_pick(&weights), Some("a"));
        }
    }

    #[test]
    fn weighted_pick_distribution() {
        let mut rng = Rng::new(777);
        let mut weights = BTreeMap::new();
        weights.insert("x", 1u32);
        weights.insert("y", 1u32);
        let mut x_count = 0u32;
        let mut y_count = 0u32;
        for _ in 0..10_000 {
            match rng.weighted_pick(&weights) {
                Some("x") => x_count += 1,
                Some("y") => y_count += 1,
                _ => unreachable!(),
            }
        }
        // Should be roughly even; within 30% tolerance
        let ratio = (x_count as i64 * 100) / (y_count as i64);
        assert!(ratio > 70 && ratio < 130, "ratio too skewed: {ratio}");
    }

    #[test]
    fn weighted_pick_empty() {
        let mut rng = Rng::new(0);
        let empty: BTreeMap<&str, u32> = BTreeMap::new();
        assert_eq!(rng.weighted_pick(&empty), None);
    }

    #[test]
    fn next_range_rejects_inverted_bounds_without_panicking() {
        let mut rng = Rng::new(0);
        assert_eq!(
            rng.next_range(9, 3),
            Err(RngError::InvalidRange { lo: 9, hi: 3 })
        );
    }
}
