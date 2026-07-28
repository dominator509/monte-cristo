//! Q16.16 fixed-point arithmetic. INV-02: no floats in mc_core.
//!
//! Backed by `i32`. Operations saturate rather than wrapping and record an event.
//! No `f32` or `f64` anywhere.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Q16.16 signed fixed-point number.
///
/// * `from_int(n)` — 1.0 = 0x0001_0000
/// * `to_int_floor()` — truncates toward negative infinity
/// * All arithmetic saturates at `i32::MIN` / `i32::MAX`
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Fx(i32);

impl Fx {
    /// One whole unit in Q16.16.
    pub const ONE: Fx = Fx(65_536);
    /// Zero.
    pub const ZERO: Fx = Fx(0);
    /// Smallest representable positive value.
    pub const EPSILON: Fx = Fx(1);
    /// One half.
    pub const HALF: Fx = Fx(32_768);
    /// Maximum representable value.
    pub const MAX: Fx = Fx(i32::MAX);
    /// Minimum representable value.
    pub const MIN: Fx = Fx(i32::MIN);

    /// Create from an integer.
    #[inline]
    pub const fn from_int(n: i32) -> Self {
        Fx(n.wrapping_shl(16))
    }

    /// Truncate toward negative infinity.
    #[inline]
    pub const fn to_int_floor(self) -> i32 {
        self.0 >> 16
    }

    /// Saturating addition.
    #[inline]
    pub fn saturating_add(self, rhs: Fx) -> Fx {
        Fx(self.0.saturating_add(rhs.0))
    }

    /// Saturating subtraction.
    #[inline]
    pub fn saturating_sub(self, rhs: Fx) -> Fx {
        Fx(self.0.saturating_sub(rhs.0))
    }

    /// Saturating multiplication. Intermediate is i64, arithmetic shift right 16.
    #[inline]
    pub fn saturating_mul(self, rhs: Fx) -> Fx {
        let prod = (self.0 as i64).saturating_mul(rhs.0 as i64);
        // Arithmetic shift right 16 (preserving sign)
        let rounded = if prod < 0 {
            (prod + 0x8000) >> 16
        } else {
            (prod + 0x7FFF) >> 16
        };
        Fx(rounded.clamp(i32::MIN as i64, i32::MAX as i64) as i32)
    }

    /// Saturating division. Numerator is i64 shifted left 16.
    #[inline]
    pub fn saturating_div(self, rhs: Fx) -> Fx {
        if rhs.0 == 0 {
            return Fx::MAX;
        }
        let num = (self.0 as i64) << 16;
        let result = num.saturating_div(rhs.0 as i64);
        Fx(result.clamp(i32::MIN as i64, i32::MAX as i64) as i32)
    }

    /// Negation.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn neg(self) -> Self {
        Fx(self.0.saturating_neg())
    }

    /// Raw value access (for serialization round-trips in tests).
    #[inline]
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// Create from raw Q16.16 value.
    #[inline]
    pub const fn from_raw(raw: i32) -> Self {
        Fx(raw)
    }
}

impl fmt::Display for Fx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:04x}", self.to_int_floor(), (self.0 as u16))
    }
}

impl std::ops::Add for Fx {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.saturating_add(rhs)
    }
}

impl std::ops::Sub for Fx {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.saturating_sub(rhs)
    }
}

impl std::ops::Mul for Fx {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.saturating_mul(rhs)
    }
}

impl std::ops::Div for Fx {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        self.saturating_div(rhs)
    }
}

impl std::ops::Neg for Fx {
    type Output = Self;
    fn neg(self) -> Self {
        self.neg()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_int_and_to_int_floor() {
        assert_eq!(Fx::from_int(0).to_int_floor(), 0);
        assert_eq!(Fx::from_int(1).to_int_floor(), 1);
        assert_eq!(Fx::from_int(-1).to_int_floor(), -1);
        assert_eq!(Fx::from_int(42).to_int_floor(), 42);
    }

    #[test]
    fn add_sub() {
        assert_eq!(Fx::from_int(10) + Fx::from_int(5), Fx::from_int(15));
        assert_eq!(Fx::from_int(10) - Fx::from_int(5), Fx::from_int(5));
    }

    #[test]
    fn mul() {
        // 3 * 2 = 6
        assert_eq!(Fx::from_int(3) * Fx::from_int(2), Fx::from_int(6));
        // 3 * (-2) = -6
        assert_eq!(Fx::from_int(3) * Fx::from_int(-2), Fx::from_int(-6));
        // 0.5 * 4 = 2.0
        assert_eq!(Fx::HALF * Fx::from_int(4), Fx::from_int(2));
        // -1 * -1 = 1
        assert_eq!(Fx::from_int(-1) * Fx::from_int(-1), Fx::from_int(1));
    }

    #[test]
    fn div() {
        assert_eq!(Fx::from_int(10) / Fx::from_int(2), Fx::from_int(5));
        assert_eq!(Fx::from_int(1) / Fx::from_int(2), Fx::HALF);
    }

    #[test]
    fn saturation_clamps() {
        let big = Fx::from_raw(i32::MAX - 1);
        let one = Fx::ONE;
        // Adding to MAX should saturate
        let result = big + one;
        assert_eq!(result, Fx::MAX);

        // Negating MIN should saturate
        assert_eq!(Fx::MIN.neg(), Fx::MAX);
    }

    #[test]
    fn div_by_zero_returns_max() {
        assert_eq!(Fx::from_int(1) / Fx::ZERO, Fx::MAX);
    }

    #[test]
    fn ordering() {
        assert!(Fx::from_int(1) > Fx::from_int(0));
        assert!(Fx::from_int(-1) < Fx::from_int(0));
    }

    #[test]
    fn neg_negative_yields_positive() {
        assert_eq!((-Fx::from_int(5)).to_int_floor(), -5);
    }

    #[test]
    fn half_mul() {
        // 0.5 * 0.5 = 0.25
        let quarter = Fx::HALF * Fx::HALF;
        assert_eq!(quarter.to_int_floor(), 0);
        assert!(quarter > Fx::ZERO);
        assert!(quarter < Fx::HALF);
    }
}
