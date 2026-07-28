//! Battle interface: ATB gauges, target selection, tech/item lists.
//!
//! SPEC-004 section 4. Rendered as an overlay when the core indicates battle state.

use macroquad::prelude::*;

/// Draw the battle interface overlay.
pub fn draw_battle_interface(_tick: u64) {
    // M4 fills this in with full ATB, targeting, tech/item lists.
    // For M1, this is a placeholder that draws nothing.
}
