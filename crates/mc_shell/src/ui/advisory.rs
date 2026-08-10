//! Content advisory screen (SPEC-004 section 6).
//!
//! Shown before the title screen on first run. Dismissible, remembered in settings.
//! States that the game depicts suicide and the death of a child, faithfully to
//! the source novel. Re-readable from settings.

use macroquad::prelude::*;

/// The copy shown before the title screen. Keeping it as data makes the
/// safety-critical wording reviewable and testable without opening a window.
pub const ADVISORY_LINES: &[&str] = &[
    "CONTENT ADVISORY",
    "This game depicts suicide and the death of a child.",
    "These events are presented faithfully to Alexandre Dumas's novel.",
    "Press Z / ENTER / SPACE to continue.",
];

/// Draw the content advisory screen. Returns true if acknowledged.
pub fn draw_advisory_screen(tick: u64) -> bool {
    let pulse = if (tick / 30) % 2 == 0 { 1.0 } else { 0.82 };
    let paper = Color::new(0.92, 0.78, 0.42, 1.0);
    let text = Color::new(1.0, 0.96, 0.84, 1.0);

    clear_background(Color::new(0.04, 0.03, 0.08, 1.0));
    draw_rectangle(12.0, 28.0, 232.0, 168.0, Color::new(0.08, 0.05, 0.12, 1.0));
    draw_rectangle_lines(12.0, 28.0, 232.0, 168.0, 2.0, paper);
    draw_text(ADVISORY_LINES[0], 28.0, 56.0, 18.0, paper);
    draw_text(ADVISORY_LINES[1], 28.0, 92.0, 10.0, text);
    draw_text(ADVISORY_LINES[2], 28.0, 112.0, 10.0, text);
    draw_text(
        ADVISORY_LINES[3],
        28.0,
        168.0,
        11.0,
        Color::new(paper.r, paper.g, paper.b, pulse),
    );

    is_key_pressed(KeyCode::Z) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space)
}
