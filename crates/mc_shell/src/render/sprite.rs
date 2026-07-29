//! Sprite rendering: field, battle, and Confidence portraits.
//!
//! SPEC-004 section 3. Sizes: 24x32 field, 48x64 battle, 64x80 Confidence portraits.
//! 22 characters carry portrait sets with 8 expression frames.

use macroquad::prelude::*;

/// Sprite size for field characters.
pub const FIELD_SPRITE_W: u16 = 24;
pub const FIELD_SPRITE_H: u16 = 32;

/// Sprite size for battle characters.
pub const BATTLE_SPRITE_W: u16 = 48;
pub const BATTLE_SPRITE_H: u16 = 64;

/// Portrait size for Confidence scenes.
pub const PORTRAIT_W: u16 = 64;
pub const PORTRAIT_H: u16 = 80;

/// Number of expression frames per character.
pub const EXPRESSION_FRAMES: u8 = 8;

/// A single sprite instance on screen.
#[derive(Debug, Clone)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub frame: u8,
    pub flip_x: bool,
    pub flip_y: bool,
    pub visible: bool,
    pub palette_offset: u8,
}

impl Sprite {
    pub fn new(x: f32, y: f32) -> Self {
        Sprite {
            x,
            y,
            frame: 0,
            flip_x: false,
            flip_y: false,
            visible: true,
            palette_offset: 0,
        }
    }
}
