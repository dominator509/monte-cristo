//! Audio subsystem: 8-channel sample-based playback, 34 tracks.
//!
//! SPEC-004 section 9. Act-scoped music with scene override.
//! Audio never affects state; muted and loud runs produce identical hashes.

use macroquad::prelude::*;

/// Number of audio channels.
pub const CHANNELS: u8 = 8;

/// Number of tracks in the soundtrack.
pub const TRACKS: u8 = 34;

/// The audio state.
pub struct AudioState {
    /// Whether audio is enabled (false in headless mode).
    pub enabled: bool,
    /// Master volume 0..100.
    pub volume: u8,
    /// Current track index.
    pub current_track: Option<u8>,
}

impl AudioState {
    pub fn new(enabled: bool) -> Self {
        AudioState {
            enabled,
            volume: 80,
            current_track: None,
        }
    }

    /// Update the audio state from the current tick/view.
    pub fn update(&mut self, _tick: u64, _act: mc_core::world::Act) {
        // M6 fills this in with act-scoped music selection and scene overrides.
    }
}

impl Default for AudioState {
    fn default() -> Self {
        Self::new(true)
    }
}
