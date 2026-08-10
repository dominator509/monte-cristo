//! Menu screens: party, curriculum, inventory, Web of Debt, ledger, settings,
//! plus Act II calendar and Act VI season clock.
//!
//! SPEC-004 section 5.

use macroquad::prelude::*;
use mc_core::command::StateView;

const INK: Color = Color::new(0.04, 0.03, 0.08, 0.96);
const PAPER: Color = Color::new(0.92, 0.78, 0.42, 1.0);
const TEXT: Color = Color::new(1.0, 0.96, 0.84, 1.0);

/// Draw the menu overlay for the current screen.
pub fn draw_menu_screen(_tick: u64) {
    draw_rectangle(18.0, 20.0, 220.0, 184.0, INK);
    draw_rectangle_lines(18.0, 20.0, 220.0, 184.0, 2.0, PAPER);
    draw_rectangle_lines(
        23.0,
        25.0,
        210.0,
        174.0,
        1.0,
        Color::new(0.45, 0.25, 0.35, 1.0),
    );
    draw_text("THE COUNT OF MONTE CRISTO", 31.0, 45.0, 14.0, PAPER);
    for (row, label) in ["Party", "Inventory", "Web of Debt", "Ledger", "Settings"]
        .iter()
        .enumerate()
    {
        let y = 73.0 + row as f32 * 22.0;
        draw_text(if row == 0 { ">" } else { "·" }, 42.0, y, 16.0, PAPER);
        draw_text(label, 58.0, y, 14.0, TEXT);
    }
    draw_text(
        "Z / ENTER  SELECT",
        40.0,
        184.0,
        10.0,
        Color::new(0.72, 0.68, 0.58, 1.0),
    );
    draw_text(
        "X / ESC  CLOSE",
        137.0,
        184.0,
        10.0,
        Color::new(0.72, 0.68, 0.58, 1.0),
    );
}

/// Draw the always-visible field HUD from the authoritative state projection.
pub fn draw_field_hud(view: &StateView<'_>, high_contrast: bool) {
    let panel = if high_contrast {
        Color::new(0.0, 0.0, 0.0, 0.94)
    } else {
        INK
    };
    let accent = if high_contrast { WHITE } else { PAPER };
    draw_rectangle(6.0, 6.0, 116.0, 34.0, panel);
    draw_rectangle_lines(6.0, 6.0, 116.0, 34.0, 1.0, accent);
    draw_text("EDMOND", 12.0, 19.0, 10.0, accent);
    draw_text(
        &format!(
            "HP {:>3}/{:<3}",
            view.party.active[0].hp, view.party.active[0].max_hp
        ),
        12.0,
        31.0,
        9.0,
        TEXT,
    );
    draw_rectangle(72.0, 25.0, 42.0, 4.0, Color::new(0.18, 0.10, 0.14, 1.0));
    draw_rectangle(72.0, 25.0, 42.0, 4.0, Color::new(0.30, 0.76, 0.46, 1.0));
    draw_text(
        &format!("ACT {}", act_label(view.act)),
        169.0,
        18.0,
        10.0,
        accent,
    );
    draw_text(
        &format!("T{:05}", view.tick.min(99_999)),
        196.0,
        31.0,
        9.0,
        TEXT,
    );
}

fn act_label(act: mc_core::world::Act) -> &'static str {
    match act {
        mc_core::world::Act::ActIMarseille => "I",
        mc_core::world::Act::ActIIIf => "II",
        mc_core::world::Act::ActIIIMonteCristo => "III",
        mc_core::world::Act::ActIVRome => "IV",
        mc_core::world::Act::ActVParis => "V",
        mc_core::world::Act::ActVIParis => "VI",
        mc_core::world::Act::ActVIIFinal => "VII",
    }
}
