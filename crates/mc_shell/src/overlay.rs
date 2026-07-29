//! Debug overlay (behind `debug-overlay` feature).
//!
//! Renders a read-only overlay on top of the game screen showing
//! internal state: tick, region, flags, hash, memory, FPS.
//! Reads StateView only — cannot alter state hash (INV-04).

use mc_core::command::StateView;

/// Overlay state — all fields are read-only copies.
#[derive(Debug, Clone)]
pub struct DebugOverlay {
    pub visible: bool,
    pub tick: u64,
    pub region: String,
    pub flags_count: usize,
    pub state_hash: String,
    pub fps: f64,
    pub memory_mb: f64,
}

impl DebugOverlay {
    /// Create a new overlay (initially hidden).
    pub fn new() -> Self {
        DebugOverlay {
            visible: false,
            tick: 0,
            region: String::new(),
            flags_count: 0,
            state_hash: String::new(),
            fps: 0.0,
            memory_mb: 0.0,
        }
    }

    /// Update overlay data from a StateView.
    /// This is a pure read — no mutation of core state (INV-04).
    pub fn update(&mut self, view: &StateView) {
        self.tick = view.tick;
        self.region = format!("{:?}", view.region);
        self.flags_count = view.flags.raw_bits().count_ones() as usize;
        self.state_hash = view.state_hash.map(|h| hex_fmt(&h)).unwrap_or_default();
    }

    /// Toggle visibility.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

fn hex_fmt(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_starts_hidden() {
        let overlay = DebugOverlay::new();
        assert!(!overlay.visible, "overlay should start hidden");
    }

    #[test]
    fn toggle_visibility() {
        let mut overlay = DebugOverlay::new();
        overlay.toggle();
        assert!(overlay.visible, "toggle should make visible");
        overlay.toggle();
        assert!(!overlay.visible, "second toggle should hide");
    }
}
