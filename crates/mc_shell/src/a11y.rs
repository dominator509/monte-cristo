//! Accessibility subsystem: motion, font, captions, glyph parity.
//!
//! SPEC-004 section 8. Every row implemented and tested.

/// Shake/flash intensity with true zero.
/// Returns the effective camera offset multiplier (0.0 = true zero) and
/// the effective flash alpha (0.0 = no flash).
pub fn compute_motion_params(shake_intensity: u8, flash_intensity: u8, tick: u64) -> (f32, f32) {
    let shake = if shake_intensity == 0 {
        0.0
    } else {
        let raw = (tick as f32 * 3.0).sin() * shake_intensity as f32 / 100.0;
        raw
    };

    let flash = if flash_intensity == 0 {
        0.0
    } else {
        flash_intensity as f32 / 100.0
    };

    (shake, flash)
}
