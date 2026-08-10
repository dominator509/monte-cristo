//! Title screen presentation and first-run entry choices.
//!
//! SPEC-004 section 4. The title is shown after the content advisory in the
//! windowed shell and offers New Game or Continue without touching core state.

use macroquad::prelude::*;

/// Draw the deterministic title screen.
pub fn draw_title_screen(selected: usize) {
    clear_background(Color::new(0.03, 0.02, 0.07, 1.0));
    for (index, colour) in [
        Color::new(0.04, 0.04, 0.12, 1.0),
        Color::new(0.07, 0.04, 0.12, 1.0),
        Color::new(0.12, 0.05, 0.10, 1.0),
        Color::new(0.18, 0.07, 0.08, 1.0),
    ]
    .into_iter()
    .enumerate()
    {
        draw_rectangle(0.0, index as f32 * 56.0, 256.0, 56.0, colour);
    }

    // A small moon and horizon silhouette give the title card a distinct
    // identity while remaining crisp at the 256x224 internal resolution.
    draw_circle(204.0, 48.0, 20.0, Color::new(0.92, 0.78, 0.42, 1.0));
    draw_circle(214.0, 42.0, 20.0, Color::new(0.07, 0.04, 0.12, 1.0));
    draw_rectangle(0.0, 142.0, 256.0, 82.0, Color::new(0.03, 0.02, 0.05, 1.0));
    for x in (0..256).step_by(16) {
        let height = 8.0 + ((x * 7) % 17) as f32;
        draw_rectangle(
            x as f32,
            142.0 - height,
            12.0,
            height,
            Color::new(0.05, 0.03, 0.08, 1.0),
        );
    }

    draw_text(
        "THE COUNT",
        34.0,
        68.0,
        25.0,
        Color::new(1.0, 0.96, 0.84, 1.0),
    );
    draw_text(
        "OF MONTE CRISTO",
        27.0,
        97.0,
        18.0,
        Color::new(0.92, 0.78, 0.42, 1.0),
    );
    draw_line(
        32.0,
        108.0,
        224.0,
        108.0,
        1.0,
        Color::new(0.72, 0.48, 0.46, 1.0),
    );

    for (index, label) in ["NEW GAME", "CONTINUE"].iter().enumerate() {
        let y = 170.0 + index as f32 * 20.0;
        let active = index == selected.min(1);
        draw_text(
            if active { ">" } else { "·" },
            74.0,
            y,
            16.0,
            Color::new(0.92, 0.78, 0.42, 1.0),
        );
        draw_text(
            label,
            94.0,
            y,
            13.0,
            if active {
                Color::new(0.92, 0.78, 0.42, 1.0)
            } else {
                Color::new(1.0, 0.96, 0.84, 1.0)
            },
        );
    }
    draw_text(
        "Z / ENTER  SELECT",
        70.0,
        214.0,
        9.0,
        Color::new(0.72, 0.68, 0.58, 1.0),
    );
}
