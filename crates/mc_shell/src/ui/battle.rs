//! Battle interface: ATB gauges, target selection, tech/item lists.
//!
//! SPEC-004 section 4. Rendered as an overlay when the core indicates battle state.

use macroquad::prelude::*;

/// Draw the battle interface overlay.
pub fn draw_battle_interface(_tick: u64) {
    draw_rectangle(8.0, 148.0, 240.0, 68.0, Color::new(0.04, 0.03, 0.08, 0.96));
    draw_rectangle_lines(
        8.0,
        148.0,
        240.0,
        68.0,
        2.0,
        Color::new(0.92, 0.78, 0.42, 1.0),
    );
    draw_text("ATTACK", 22.0, 174.0, 13.0, WHITE);
    draw_text("TECH", 22.0, 192.0, 13.0, Color::new(0.72, 0.68, 0.58, 1.0));
    draw_text("ITEM", 22.0, 210.0, 13.0, Color::new(0.72, 0.68, 0.58, 1.0));
    draw_text("ATB", 138.0, 174.0, 11.0, Color::new(0.92, 0.78, 0.42, 1.0));
    draw_rectangle(164.0, 166.0, 68.0, 6.0, Color::new(0.18, 0.10, 0.14, 1.0));
    draw_rectangle(164.0, 166.0, 44.0, 6.0, Color::new(0.35, 0.70, 0.90, 1.0));
    draw_text("TARGET  FERNAND", 138.0, 194.0, 10.0, WHITE);
}
