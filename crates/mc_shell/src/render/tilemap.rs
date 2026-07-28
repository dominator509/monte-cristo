//! Tilemap rendering: two scrolling background layers plus one overlay.
//!
//! SPEC-004 section 1. Tiles are 16x16 pixels on a 256x224 grid (16x14 tiles).
//! Two background layers scroll independently; one overlay layer is fixed.

use macroquad::prelude::*;

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
}

impl Default for Tilemap {
    fn default() -> Self {
        Self::new()
    }
}
