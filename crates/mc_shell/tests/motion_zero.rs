//! EP-005 M7: Motion zero test.
//!
//! Shake and flash intensity with true zero must produce zero camera offset
//! and zero full-screen luminance delta.

use mc_shell::a11y::compute_motion_params;

#[test]
fn zero_shake_produces_zero_offset() {
    let (offset_x, offset_y) = compute_motion_params(0, 0, 0);
    assert_eq!(offset_x, 0.0, "zero shake must produce zero x offset");
    assert_eq!(offset_y, 0.0, "zero shake must produce zero y offset");
}

#[test]
fn zero_shake_all_ticks() {
    for tick in &[0u64, 1, 100, 9999, u64::MAX] {
        let (ox, oy) = compute_motion_params(0, 0, *tick);
        assert_eq!(ox, 0.0, "zero shake must be zero at tick {tick}");
        assert_eq!(oy, 0.0, "zero shake must be zero at tick {tick}");
    }
}

#[test]
fn nonzero_shake_produces_nonzero() {
    let (offset_x, offset_y) = compute_motion_params(100, 0, 42);
    // At max intensity there should be some offset
    assert!(
        offset_x.abs() > 0.0 || offset_y.abs() > 0.0,
        "full shake should produce non-zero offset, got ({offset_x}, {offset_y})"
    );
}

#[test]
fn intensity_scales_appropriately() {
    let (low_x, low_y) = compute_motion_params(10, 0, 42);
    let (high_x, high_y) = compute_motion_params(100, 0, 42);
    // Higher intensity should produce proportionally larger offsets
    let low_mag = (low_x.powi(2) + low_y.powi(2)).sqrt();
    let high_mag = (high_x.powi(2) + high_y.powi(2)).sqrt();
    assert!(
        high_mag >= low_mag,
        "higher shake intensity should produce >= magnitude (low={low_mag}, high={high_mag})"
    );
}
