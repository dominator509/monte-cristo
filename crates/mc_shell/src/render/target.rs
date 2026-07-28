//! 256x224 render target with nearest-neighbour integer scaling.
//!
//! SPEC-004 section 1. Internal resolution is 256x224 with 16x16 tiles.
//! The target is scaled 1x-6x with integer scaling and letterboxing.

use macroquad::prelude::*;
use macroquad::texture::{render_target, RenderTarget};

/// Internal resolution: 256 wide.
pub const INTERNAL_WIDTH: u16 = 256;
/// Internal resolution: 224 tall.
pub const INTERNAL_HEIGHT: u16 = 224;

/// Tile size in pixels.
pub const TILE_SIZE: u16 = 16;

/// Maximum integer scale factor.
pub const MAX_SCALE: u16 = 6;

/// The render target and its scaling parameters.
pub struct ShellRenderTarget {
    /// The offscreen render target at 256x224.
    pub target: RenderTarget,
    /// The integer scale factor (1..=6).
    pub scale: u16,
    /// Horizontal letterbox offset (pixels) on the window.
    pub offset_x: f32,
    /// Vertical letterbox offset (pixels) on the window.
    pub offset_y: f32,
    /// The actual rendered width on screen.
    pub screen_w: f32,
    /// The actual rendered height on screen.
    pub screen_h: f32,
}

impl ShellRenderTarget {
    /// Create a new render target and compute scaling for the current window.
    pub fn new() -> Self {
        let target = render_target(INTERNAL_WIDTH as u32, INTERNAL_HEIGHT as u32);
        let (scale, offset_x, offset_y, screen_w, screen_h) = Self::compute_scale();
        ShellRenderTarget {
            target,
            scale,
            offset_x,
            offset_y,
            screen_w,
            screen_h,
        }
    }

    /// Recompute the scale factor and letterbox offsets for the current window size.
    pub fn recompute(&mut self) {
        let (scale, offset_x, offset_y, screen_w, screen_h) = Self::compute_scale();
        self.scale = scale;
        self.offset_x = offset_x;
        self.offset_y = offset_y;
        self.screen_w = screen_w;
        self.screen_h = screen_h;
    }

    /// Compute the integer scale and letterbox offsets.
    fn compute_scale() -> (u16, f32, f32, f32, f32) {
        let (win_w, win_h) = (screen_width(), screen_height());
        let scale_x = (win_w / INTERNAL_WIDTH as f32).floor() as u16;
        let scale_y = (win_h / INTERNAL_HEIGHT as f32).floor() as u16;
        let scale = scale_x.min(scale_y).min(MAX_SCALE).max(1);
        let sw = INTERNAL_WIDTH as f32 * scale as f32;
        let sh = INTERNAL_HEIGHT as f32 * scale as f32;
        let ox = ((win_w - sw) / 2.0).floor();
        let oy = ((win_h - sh) / 2.0).floor();
        (scale, ox, oy, sw, sh)
    }

    /// Set up camera to render to the internal target.
    pub fn set_camera(&self) {
        let camera = Camera2D {
            zoom: vec2(2.0 / INTERNAL_WIDTH as f32, -2.0 / INTERNAL_HEIGHT as f32),
            target: vec2(INTERNAL_WIDTH as f32 / 2.0, INTERNAL_HEIGHT as f32 / 2.0),
            render_target: Some(self.target.clone()),
            ..Default::default()
        };
        set_camera(&camera);
    }

    /// Clear the internal render target.
    pub fn clear(&self, color: Color) {
        self.set_camera();
        clear_background(color);
    }

    /// Blit the internal target to the screen with nearest-neighbour scaling.
    pub fn blit(&self) {
        set_default_camera();
        let params = DrawTextureParams {
            dest_size: Some(vec2(self.screen_w, self.screen_h)),
            ..Default::default()
        };
        draw_texture_ex(
            &self.target.texture,
            self.offset_x,
            self.offset_y,
            WHITE,
            params,
        );
    }

    /// Check if a window resize has occurred and re-scale if so.
    pub fn handle_resize(&mut self) {
        let (win_w, win_h) = (screen_width(), screen_height());
        let scale_x = (win_w / INTERNAL_WIDTH as f32).floor() as u16;
        let scale_y = (win_h / INTERNAL_HEIGHT as f32).floor() as u16;
        let new_scale = scale_x.min(scale_y).min(MAX_SCALE).max(1);
        if new_scale != self.scale {
            self.recompute();
        }
    }
}

impl Default for ShellRenderTarget {
    fn default() -> Self {
        Self::new()
    }
}
