//! EP-002 M1: Property tests for Q16.16 fixed-point arithmetic.
//!
//! Asserts over 1,000,000 generated pairs that no operation panics and that
//! saturation clamps at `i32::MAX` / `i32::MIN`.
//!
//! Run full million-case suite:
//!   PROPTEST_CASES=250000 cargo test --locked -p mc_core --test prop_fixed_point

use mc_core::fx::Fx;

proptest::proptest! {
    #[test]
    fn add_never_panics(a in proptest::prelude::any::<i32>(), b in proptest::prelude::any::<i32>()) {
        let _ = Fx::from_raw(a) + Fx::from_raw(b);
    }

    #[test]
    fn sub_never_panics(a in proptest::prelude::any::<i32>(), b in proptest::prelude::any::<i32>()) {
        let _ = Fx::from_raw(a) - Fx::from_raw(b);
    }

    #[test]
    fn mul_never_panics(a in proptest::prelude::any::<i32>(), b in proptest::prelude::any::<i32>()) {
        let _ = Fx::from_raw(a) * Fx::from_raw(b);
    }

    #[test]
    fn div_never_panics(a in proptest::prelude::any::<i32>(), b in proptest::prelude::any::<i32>()) {
        let _ = Fx::from_raw(a) / Fx::from_raw(b);
    }
}

proptest::proptest! {
    #[test]
    fn add_saturates_at_bounds(a in proptest::prelude::any::<i32>(), b in proptest::prelude::any::<i32>()) {
        let result = Fx::from_raw(a) + Fx::from_raw(b);
        let true_sum = (a as i64).saturating_add(b as i64);
        if true_sum > i32::MAX as i64 {
            assert_eq!(result.raw(), i32::MAX, "add saturates at MAX");
        } else if true_sum < i32::MIN as i64 {
            assert_eq!(result.raw(), i32::MIN, "add saturates at MIN");
        }
    }

    #[test]
    fn sub_saturates_at_bounds(a in proptest::prelude::any::<i32>(), b in proptest::prelude::any::<i32>()) {
        let result = Fx::from_raw(a) - Fx::from_raw(b);
        let true_diff = (a as i64).saturating_sub(b as i64);
        if true_diff > i32::MAX as i64 {
            assert_eq!(result.raw(), i32::MAX, "sub saturates at MAX");
        } else if true_diff < i32::MIN as i64 {
            assert_eq!(result.raw(), i32::MIN, "sub saturates at MIN");
        }
    }

    #[test]
    fn mul_saturates_at_bounds(a in proptest::prelude::any::<i32>(), b in proptest::prelude::any::<i32>()) {
        let result = Fx::from_raw(a) * Fx::from_raw(b);
        let product = (a as i64).saturating_mul(b as i64);
        let scaled_max = (i32::MAX as i64) << 16;
        let scaled_min = (i32::MIN as i64) << 16;
        if product > scaled_max {
            assert_eq!(result.raw(), i32::MAX, "mul saturates at MAX");
        } else if product < scaled_min {
            assert_eq!(result.raw(), i32::MIN, "mul saturates at MIN");
        }
    }

    #[test]
    fn div_saturates_at_bounds(a in proptest::prelude::any::<i32>(), b in proptest::prelude::any::<i32>()) {
        let result = Fx::from_raw(a) / Fx::from_raw(b);
        if b == 0 {
            assert_eq!(result.raw(), i32::MAX, "div by zero returns MAX");
        } else {
            let numerator = (a as i64) << 16;
            let scaled_max = (i32::MAX as i64) << 16;
            let scaled_min = (i32::MIN as i64) << 16;
            if numerator > scaled_max {
                assert_eq!(result.raw(), i32::MAX, "div saturates at MAX");
            } else if numerator < scaled_min {
                assert_eq!(result.raw(), i32::MIN, "div saturates at MIN");
            }
        }
    }
}

proptest::proptest! {
    #[test]
    fn from_int_roundtrip(n in proptest::prelude::any::<i16>()) {
        let fx = Fx::from_int(n as i32);
        assert_eq!(fx.to_int_floor(), n as i32);
    }

    #[test]
    fn from_raw_roundtrip(raw in proptest::prelude::any::<i32>()) {
        let fx = Fx::from_raw(raw);
        assert_eq!(fx.raw(), raw);
    }
}
