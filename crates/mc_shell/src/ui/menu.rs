//! Menu screens: party, curriculum, inventory, Web of Debt, ledger, settings,
//! plus Act II calendar and Act VI season clock.
//!
//! SPEC-004 section 5.

use crate::app::FileSelectMode;
use crate::config::{TextSpeed, ValidatedConfig};
use macroquad::prelude::*;
use mc_core::command::StateView;

const INK: Color = Color::new(0.04, 0.03, 0.08, 0.96);
const PAPER: Color = Color::new(0.92, 0.78, 0.42, 1.0);
const TEXT: Color = Color::new(1.0, 0.96, 0.84, 1.0);

/// Menu entries shown by the main shell menu.
pub const MENU_ENTRIES: &[&str] = &[
    "Party",
    "Curriculum",
    "Inventory",
    "Web of Debt",
    "Ledger",
    "Settings",
    "Save",
    "Load",
];
/// Index of the Save entry in [`MENU_ENTRIES`].
pub const MENU_SAVE_INDEX: usize = 6;
/// Index of the Load entry in [`MENU_ENTRIES`].
pub const MENU_LOAD_INDEX: usize = 7;
/// Number of save slots exposed by the file-select screen.
pub const SAVE_SLOT_COUNT: usize = 4;

/// Read-only detail views exposed from the main menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuDetail {
    Party,
    Curriculum,
    Inventory,
    WebOfDebt,
    Ledger,
    Settings,
}

/// Draw the menu overlay for the current screen.
pub fn draw_menu_screen(_tick: u64, selected: usize) {
    draw_rectangle(18.0, 14.0, 220.0, 196.0, INK);
    draw_rectangle_lines(18.0, 14.0, 220.0, 196.0, 2.0, PAPER);
    draw_rectangle_lines(
        23.0,
        19.0,
        210.0,
        186.0,
        1.0,
        Color::new(0.45, 0.25, 0.35, 1.0),
    );
    draw_text("THE COUNT OF MONTE CRISTO", 31.0, 39.0, 14.0, PAPER);
    for (row, label) in MENU_ENTRIES.iter().enumerate() {
        let y = 61.0 + row as f32 * 18.0;
        let active = row == selected.min(MENU_ENTRIES.len().saturating_sub(1));
        draw_text(if active { ">" } else { "·" }, 38.0, y, 16.0, PAPER);
        draw_text(label, 54.0, y, 13.0, if active { PAPER } else { TEXT });
    }
    draw_text(
        "Z / ENTER  SELECT",
        40.0,
        194.0,
        10.0,
        Color::new(0.72, 0.68, 0.58, 1.0),
    );
    draw_text(
        "X / ESC  CLOSE",
        142.0,
        194.0,
        10.0,
        Color::new(0.72, 0.68, 0.58, 1.0),
    );
}

/// Draw a read-only detail view from the authoritative state projection.
pub fn draw_menu_detail_screen(detail: MenuDetail, view: &StateView<'_>, config: &ValidatedConfig) {
    draw_rectangle(18.0, 14.0, 220.0, 196.0, INK);
    draw_rectangle_lines(18.0, 14.0, 220.0, 196.0, 2.0, PAPER);
    draw_rectangle_lines(
        23.0,
        19.0,
        210.0,
        186.0,
        1.0,
        Color::new(0.45, 0.25, 0.35, 1.0),
    );
    let title = match detail {
        MenuDetail::Party => "PARTY",
        MenuDetail::Curriculum => "CURRICULUM",
        MenuDetail::Inventory => "INVENTORY",
        MenuDetail::WebOfDebt => "WEB OF DEBT",
        MenuDetail::Ledger => "LEDGER",
        MenuDetail::Settings => "SETTINGS",
    };
    draw_text(title, 34.0, 46.0, 16.0, PAPER);

    match detail {
        MenuDetail::Party => draw_party_detail(view),
        MenuDetail::Curriculum => draw_curriculum_detail(view),
        MenuDetail::Inventory => draw_inventory_detail(view),
        MenuDetail::WebOfDebt => {
            draw_text("DEBTS ARE TRACKED BY STORY SCENES", 32.0, 78.0, 10.0, TEXT);
            draw_text("No outstanding entries.", 32.0, 104.0, 12.0, TEXT);
            draw_text(
                "Mercy changes the ending, not the ledger.",
                32.0,
                130.0,
                10.0,
                TEXT,
            );
        }
        MenuDetail::Ledger => {
            let resolved = view.flags.raw_bits().count_ones();
            draw_text(
                &format!("STORY FLAGS RESOLVED  {resolved:02}"),
                32.0,
                80.0,
                11.0,
                TEXT,
            );
            draw_text(
                &format!("CURRENT REGION  {}", view.region.name()),
                32.0,
                106.0,
                11.0,
                TEXT,
            );
            draw_text(
                "Progress is recorded by the replayable world.",
                32.0,
                132.0,
                10.0,
                TEXT,
            );
        }
        MenuDetail::Settings => draw_settings_detail(config),
    }

    draw_text(
        "X / ESC  BACK",
        154.0,
        194.0,
        10.0,
        Color::new(0.72, 0.68, 0.58, 1.0),
    );
}

fn draw_party_detail(view: &StateView<'_>) {
    if view.party.active.is_empty() {
        draw_text("NO ACTIVE PARTY MEMBERS", 32.0, 82.0, 11.0, TEXT);
        return;
    }
    for (row, member) in view.party.active.iter().enumerate() {
        let y = 76.0 + row as f32 * 28.0;
        draw_text(member.char_id.name(), 34.0, y, 12.0, PAPER);
        draw_text(
            &format!(
                "LV {}   HP {:>3}/{:<3}",
                member.level, member.hp, member.max_hp
            ),
            34.0,
            y + 14.0,
            10.0,
            TEXT,
        );
    }
}

fn draw_curriculum_detail(view: &StateView<'_>) {
    for (row, discipline) in mc_core::curriculum::Discipline::ALL.iter().enumerate() {
        let y = 70.0 + row as f32 * 17.0;
        draw_text(discipline.name(), 32.0, y, 10.0, TEXT);
        draw_text(
            &format!(
                "R{}  {:02} mo",
                view.curriculum.rank(*discipline),
                view.curriculum.months_for(*discipline)
            ),
            172.0,
            y,
            10.0,
            PAPER,
        );
    }
}

fn draw_inventory_detail(view: &StateView<'_>) {
    if view.inventory.items().is_empty() {
        draw_text("INVENTORY EMPTY", 32.0, 82.0, 11.0, TEXT);
        return;
    }
    for (row, (item, count)) in view.inventory.items().iter().take(7).enumerate() {
        let y = 72.0 + row as f32 * 17.0;
        draw_text(item.name(), 34.0, y, 11.0, TEXT);
        draw_text(&format!("x{count}"), 194.0, y, 11.0, PAPER);
    }
}

fn draw_settings_detail(config: &ValidatedConfig) {
    let speed = match config.text_speed {
        TextSpeed::Slow => "SLOW",
        TextSpeed::Normal => "NORMAL",
        TextSpeed::Fast => "FAST",
        TextSpeed::Instant => "INSTANT",
    };
    let rows = [
        format!(
            "HIGH CONTRAST  {}",
            if config.high_contrast { "ON" } else { "OFF" }
        ),
        format!(
            "CAPTIONS        {}",
            if config.captions_enabled { "ON" } else { "OFF" }
        ),
        format!("VOLUME          {:03}", config.volume),
        format!("TEXT SPEED      {speed}"),
    ];
    for (row, label) in rows.iter().enumerate() {
        draw_text(label, 34.0, 78.0 + row as f32 * 22.0, 11.0, TEXT);
    }
}

/// Draw the slot picker used by both Save and Load.
pub fn draw_file_select_screen(
    mode: FileSelectMode,
    selected: u8,
    occupied: &[bool; SAVE_SLOT_COUNT],
    error: Option<&str>,
) {
    let title = match mode {
        FileSelectMode::Save => "SAVE GAME",
        FileSelectMode::Load => "LOAD GAME",
    };
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
    draw_text(title, 34.0, 48.0, 16.0, PAPER);
    for (index, present) in occupied.iter().enumerate() {
        let y = 76.0 + index as f32 * 25.0;
        let active = index == selected as usize;
        draw_text(if active { ">" } else { "·" }, 38.0, y, 16.0, PAPER);
        draw_text(
            &format!("SLOT {}", index + 1),
            54.0,
            y,
            13.0,
            if active { PAPER } else { TEXT },
        );
        draw_text(
            if *present { "USED" } else { "EMPTY" },
            160.0,
            y,
            11.0,
            if *present {
                TEXT
            } else {
                Color::new(0.72, 0.68, 0.58, 1.0)
            },
        );
    }
    if let Some(error) = error {
        let clipped: String = error.chars().take(31).collect();
        draw_text(&clipped, 30.0, 178.0, 9.0, Color::new(1.0, 0.44, 0.38, 1.0));
    } else {
        draw_text(
            "Z / ENTER  CONFIRM",
            34.0,
            178.0,
            10.0,
            Color::new(0.72, 0.68, 0.58, 1.0),
        );
    }
    draw_text(
        "X / ESC  BACK",
        154.0,
        194.0,
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
