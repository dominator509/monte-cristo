//! Confidence scene presentation: 64x80 portraits, dialogue, choices.
//!
//! SPEC-004 section 4, SPEC-010. No combat affordances (hp, gauge, atb, turn_order).
//! Zero matches on grep for those terms is a validation criterion.

use macroquad::prelude::*;

/// Draw the Confidence scene overlay.
pub fn draw_confidence_scene(_tick: u64) {
    draw_rectangle(12.0, 12.0, 232.0, 200.0, Color::new(0.04, 0.03, 0.08, 0.98));
    draw_rectangle_lines(
        12.0,
        12.0,
        232.0,
        200.0,
        2.0,
        Color::new(0.92, 0.78, 0.42, 1.0),
    );
    draw_rectangle(28.0, 34.0, 64.0, 80.0, Color::new(0.18, 0.10, 0.14, 1.0));
    draw_circle(60.0, 62.0, 17.0, Color::new(0.92, 0.78, 0.60, 1.0));
    draw_rectangle(42.0, 80.0, 36.0, 34.0, Color::new(0.35, 0.28, 0.45, 1.0));
    draw_text(
        "CONFIDENCE",
        106.0,
        51.0,
        14.0,
        Color::new(0.92, 0.78, 0.42, 1.0),
    );
    draw_text("The truth waits behind the mask.", 106.0, 73.0, 10.0, WHITE);
    draw_text("> Listen", 106.0, 103.0, 12.0, WHITE);
    draw_text(
        "  Press Z / ENTER",
        106.0,
        126.0,
        10.0,
        Color::new(0.72, 0.68, 0.58, 1.0),
    );
}
