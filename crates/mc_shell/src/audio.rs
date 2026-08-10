//! Audio subsystem: 8-channel sample-based playback, 34 tracks.
//!
//! SPEC-004 section 9. Act-scoped music with optional scene override.
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
    ///
    /// The shell does not own the simulation, so this only selects a track. It
    /// never feeds audio state back into `mc_core` and therefore cannot affect
    /// replay hashes. The tick is accepted for frame-loop symmetry; scene
    /// callers that have an authored override should use `update_with_scene`.
    pub fn update(&mut self, tick: u64, act: mc_core::world::Act) {
        self.update_with_scene(tick, act, None);
    }

    /// Select an authored scene override, falling back to the act theme.
    pub fn update_with_scene(
        &mut self,
        _tick: u64,
        act: mc_core::world::Act,
        scene_override: Option<u8>,
    ) {
        if !self.enabled {
            self.current_track = None;
            return;
        }

        let track = scene_override
            .map(|index| index % TRACKS)
            .unwrap_or_else(|| act_track(act));
        self.current_track = Some(track);
    }
}

/// Stable soundtrack slot for each authored act.
fn act_track(act: mc_core::world::Act) -> u8 {
    match act {
        mc_core::world::Act::ActIMarseille => 0,
        mc_core::world::Act::ActIIIf => 5,
        mc_core::world::Act::ActIIIMonteCristo => 10,
        mc_core::world::Act::ActIVRome => 15,
        mc_core::world::Act::ActVParis => 20,
        mc_core::world::Act::ActVIParis => 25,
        mc_core::world::Act::ActVIIFinal => 30,
    }
}

impl Default for AudioState {
    fn default() -> Self {
        Self::new(true)
    }
}
