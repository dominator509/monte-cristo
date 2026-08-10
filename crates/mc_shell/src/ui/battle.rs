//! Battle interface: ATB gauges, target selection, tech/item lists.
//!
//! SPEC-004 section 4. Rendered as an overlay when the core indicates battle state.

use macroquad::prelude::*;
use mc_core::battle::Affiliation;
use mc_core::command::StateView;

/// Draw the battle interface overlay from the authoritative battle projection.
pub fn draw_battle_interface(view: &StateView<'_>) {
    let panel = Color::new(0.04, 0.03, 0.08, 0.96);
    let accent = Color::new(0.92, 0.78, 0.42, 1.0);
    draw_rectangle(8.0, 132.0, 240.0, 84.0, panel);
    draw_rectangle_lines(8.0, 132.0, 240.0, 84.0, 2.0, accent);
    draw_text("BATTLE", 18.0, 147.0, 12.0, accent);

    let Some(battle) = view.battle else {
        draw_text("NO ACTIVE ENCOUNTER", 18.0, 170.0, 11.0, WHITE);
        return;
    };
    let mut party_row = 0usize;
    let mut enemy_row = 0usize;
    for combatant in &battle.combatants {
        let (x, y) = match combatant.affiliation {
            Affiliation::Party => {
                let row = party_row;
                party_row += 1;
                (18.0, 160.0 + row as f32 * 17.0)
            }
            Affiliation::Enemy => {
                let row = enemy_row;
                enemy_row += 1;
                (136.0, 160.0 + row as f32 * 17.0)
            }
        };
        let hp = format!(
            "{} {}/{}",
            combatant.name,
            combatant.hp.to_int_floor(),
            combatant.max_hp.to_int_floor()
        );
        draw_text(
            &hp,
            x,
            y,
            9.0,
            if combatant.is_alive() {
                WHITE
            } else {
                Color::new(0.55, 0.45, 0.45, 1.0)
            },
        );
    }
    if let Some(party) = battle
        .combatants
        .iter()
        .find(|c| c.affiliation == Affiliation::Party)
    {
        let ratio = (party.atb.value.raw().max(0) as f32 / mc_core::fx::Fx::ONE.raw() as f32)
            .clamp(0.0, 1.0);
        draw_text("ATB", 18.0, 211.0, 9.0, accent);
        draw_rectangle(42.0, 205.0, 58.0, 5.0, Color::new(0.18, 0.10, 0.14, 1.0));
        draw_rectangle(
            42.0,
            205.0,
            58.0 * ratio,
            5.0,
            Color::new(0.35, 0.70, 0.90, 1.0),
        );
    }
    draw_text("Z  ATTACK", 136.0, 211.0, 9.0, accent);
}
