//! Text rendering: dialogue boxes, menus, speed control, font management.
//!
//! SPEC-004 section 8: four text speeds including instant, font metrics.

use macroquad::prelude::*;

/// Available text speeds (SPEC-004 section 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextSpeed {
    Slow,
    Normal,
    Fast,
    Instant,
}

/// Character delay per text speed in seconds.
pub fn text_speed_delay(speed: TextSpeed) -> f64 {
    match speed {
        TextSpeed::Slow => 0.08,
        TextSpeed::Normal => 0.04,
        TextSpeed::Fast => 0.015,
        TextSpeed::Instant => 0.0,
    }
}

/// Draw a text box with the given text, at the given position and size.
pub fn draw_text_box(x: f32, y: f32, w: f32, h: f32, _text: &str, _reveal: usize) {
    // Draw background
    draw_rectangle(x, y, w, h, Color::from_rgba(8, 8, 16, 230));
    draw_rectangle_lines(x, y, w, h, 1.0, Color::from_rgba(20, 12, 24, 255));

    // M7 fills in with actual font-based text rendering.
}
