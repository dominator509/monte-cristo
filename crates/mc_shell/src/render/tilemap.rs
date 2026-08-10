//! Tilemap rendering: two scrolling background layers plus one overlay.
//!
//! SPEC-004 section 1. Tiles are 16x16 pixels on a 256x224 grid (16x14 tiles).
//! Two background layers scroll independently; one overlay layer is fixed.

use macroquad::prelude::*;
use mc_core::ids::RegionId;
use mc_core::world::Act;

/// Tiles across (256 / 16).
pub const TILES_X: u16 = 16;
/// Tiles down (224 / 16 = 14).
pub const TILES_Y: u16 = 14;

/// Maximum map width in tiles.
pub const MAP_WIDTH: u16 = 64;
/// Maximum map height in tiles.
pub const MAP_HEIGHT: u64 = 64;

/// A tile identifier: index into the tileset.
pub type TileId = u16;

/// A single layer of the tilemap.
#[derive(Debug, Clone)]
pub struct TileLayer {
    /// The tile grid (MAP_WIDTH x MAP_HEIGHT).
    pub tiles: Vec<TileId>,
    /// Width of the map in tiles (<= MAP_WIDTH).
    pub width: u16,
    /// Height of the map in tiles (<= MAP_HEIGHT).
    pub height: u16,
    /// Scroll offset in pixels (horizontal).
    pub scroll_x: f32,
    /// Scroll offset in pixels (vertical).
    pub scroll_y: f32,
}

impl TileLayer {
    pub fn new(width: u16, height: u16) -> Self {
        TileLayer {
            tiles: vec![0; (width as usize) * (height as usize)],
            width,
            height,
            scroll_x: 0.0,
            scroll_y: 0.0,
        }
    }

    /// Get a tile at the given (x, y) tile coordinates.
    pub fn get_tile(&self, x: u16, y: u16) -> TileId {
        if x >= self.width || y >= self.height {
            return 0;
        }
        self.tiles[(y as usize) * (self.width as usize) + (x as usize)]
    }

    /// Set a tile at the given (x, y) tile coordinates.
    pub fn set_tile(&mut self, x: u16, y: u16, tile: TileId) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y as usize) * (self.width as usize) + (x as usize);
        if idx < self.tiles.len() {
            self.tiles[idx] = tile;
        }
    }
}

/// The complete tilemap: two scrolling background layers plus one overlay.
#[derive(Debug, Clone)]
pub struct Tilemap {
    /// Layer 0: furthest background.
    pub layer0: TileLayer,
    /// Layer 1: midground.
    pub layer1: TileLayer,
    /// Overlay: fixed-position UI layer.
    pub overlay: TileLayer,
}

impl Tilemap {
    pub fn new() -> Self {
        Tilemap {
            layer0: TileLayer::new(TILES_X, TILES_Y),
            layer1: TileLayer::new(TILES_X, TILES_Y),
            overlay: TileLayer::new(TILES_X, TILES_Y),
        }
    }

    /// Build a deterministic scene composition from the authoritative act and
    /// region. This is presentation-only: it never feeds back into `mc_core`
    /// or changes replay state, but it gives each authored location a stable
    /// visual language instead of a startup checkerboard.
    pub fn for_scene(act: Act, region: RegionId) -> Self {
        let mut map = Self::new();
        let seed = region.raw().wrapping_add((act as u16).wrapping_mul(19));
        let ground = 1 + seed % 3;

        for y in 0..TILES_Y {
            for x in 0..TILES_X {
                let wave = (x.wrapping_mul(5) + y.wrapping_mul(3) + seed) % 11;
                let ground_tile = if wave < 6 { ground } else { ground + 1 };
                map.layer0.set_tile(x, y, ground_tile);

                let detail = (x.wrapping_mul(7) + y.wrapping_mul(11) + seed) % 13;
                let overlay_tile = if detail == 0 {
                    3
                } else if detail == 1 || detail == 2 {
                    2
                } else {
                    0
                };
                map.layer1.set_tile(x, y, overlay_tile);

                // The fixed overlay carries sparse landmark accents. Keeping
                // these in the presentation tilemap (rather than deriving
                // them from wall-clock state) makes the facelift deterministic
                // while giving each authored location a readable silhouette.
                let landmark = (x.wrapping_mul(13) + y.wrapping_mul(17) + seed) % 29;
                let overlay_tile = match landmark {
                    0 => 1,     // a short foreground rail / pier
                    1 | 2 => 2, // a shrub or cypress cluster
                    3 => 3,     // a glint / hanging lamp
                    _ => 0,
                };
                map.overlay.set_tile(x, y, overlay_tile);
            }
        }
        map
    }

    /// Return deterministic parallax offsets for the two scrolling background
    /// layers. The offsets are derived from the authoritative replay tick, not
    /// wall-clock time, so a visual frame can never affect simulation or make
    /// two replays render different motion (INV-02, INV-04).
    pub fn scroll_offsets(&self, tick: u64) -> (f32, f32) {
        let far_width = (self.layer0.width.max(1) as f32) * 16.0;
        let near_width = (self.layer1.width.max(1) as f32) * 16.0;
        let far = -((tick % far_width as u64) as f32) / 32.0;
        let near = -((tick % near_width as u64) as f32) / 8.0;
        (far.rem_euclid(far_width), near.rem_euclid(near_width))
    }
}

impl Default for Tilemap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_tilemap_is_region_specific_and_deterministic() {
        let marseille = Tilemap::for_scene(Act::ActIMarseille, RegionId::R01_MARSEILLE);
        let repeat = Tilemap::for_scene(Act::ActIMarseille, RegionId::R01_MARSEILLE);
        let if_map = Tilemap::for_scene(Act::ActIIIf, RegionId::R02_CHATEAU_DIF);
        assert_eq!(marseille.layer0.tiles, repeat.layer0.tiles);
        assert_ne!(marseille.layer0.tiles, if_map.layer0.tiles);
    }

    #[test]
    fn parallax_offsets_are_replay_deterministic_and_bounded() {
        let map = Tilemap::for_scene(Act::ActIMarseille, RegionId::R01_MARSEILLE);
        let first = map.scroll_offsets(1234);
        assert_eq!(first, map.scroll_offsets(1234));
        assert_ne!(first, map.scroll_offsets(1235));
        let far_width = map.layer0.width as f32 * 16.0;
        let near_width = map.layer1.width as f32 * 16.0;
        assert!(first.0 >= 0.0 && first.0 < far_width);
        assert!(first.1 >= 0.0 && first.1 < near_width);
    }
}
