//! Affine (Mode-7 equivalent) layer.
//!
//! SPEC-004 section 1. Used for the Pharaon under sail, Mediterranean map,
//! Monte Cristo grotto reveal, and Chateau d'If flyover.

use macroquad::prelude::*;

/// Affine transformation matrix for the layer.
#[derive(Debug, Clone, Copy)]
pub struct AffineMatrix {
    pub a: f32, // scale-x, also horizontal skew
    pub b: f32, // horizontal skew
    pub c: f32, // vertical skew
    pub d: f32, // scale-y, also vertical skew
    pub tx: f32, // translate-x
    pub ty: f32, // translate-y
}

impl AffineMatrix {
    pub fn identity() -> Self {
        AffineMatrix {
            a: 1.0, b: 0.0,
            c: 0.0, d: 1.0,
            tx: 0.0, ty: 0.0,
        }
    }

    /// Create a rotation + scale matrix around a center point.
    pub fn rotate_scale(angle: f32, scale: f32, cx: f32, cy: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        AffineMatrix {
            a: cos * scale,
            b: -sin * scale,
            c: sin * scale,
            d: cos * scale,
            tx: cx * (1.0 - cos * scale) + cy * (sin * scale),
            ty: cy * (1.0 - cos * scale) - cx * (sin * scale),
        }
    }
}

impl Default for AffineMatrix {
    fn default() -> Self {
        Self::identity()
    }
}

/// The affine layer state for the current frame.
#[derive(Debug, Clone)]
pub struct AffineLayer {
    pub matrix: AffineMatrix,
    pub texture_id: Option<u64>,
    pub visible: bool,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

impl AffineLayer {
    pub fn new() -> Self {
        AffineLayer {
            matrix: AffineMatrix::identity(),
            texture_id: None,
            visible: false,
            scroll_x: 0.0,
            scroll_y: 0.0,
        }
    }
}

impl Default for AffineLayer {
    fn default() -> Self {
        Self::new()
    }
}
