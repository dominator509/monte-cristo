//! Content advisory screen (SPEC-004 section 6).
//!
//! Shown before the title screen on first run. Dismissible, remembered in settings.
//! States that the game depicts suicide and the death of a child, faithfully to
//! the source novel. Re-readable from settings.

use macroquad::prelude::*;

/// Draw the content advisory screen. Returns true if acknowledged.
pub fn draw_advisory_screen(_tick: u64) -> bool {
    // M8 fills this in with the full advisory screen.
    // For now, return true (already acknowledged) so the game can proceed.
    true
}
