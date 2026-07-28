//! Confidence scene presentation: 64x80 portraits, dialogue, choices.
//!
//! SPEC-004 section 4, SPEC-010. No combat affordances (hp, gauge, atb, turn_order).
//! Zero matches on grep for those terms is a validation criterion.

use macroquad::prelude::*;

/// Draw the Confidence scene overlay.
pub fn draw_confidence_scene(_tick: u64) {
    // M5 fills this in with full portrait + dialogue + choices.
}
