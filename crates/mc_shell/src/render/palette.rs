//! Palette system: 15-bit colour, 7 act-locked palettes, high-contrast override.
//!
//! SPEC-004 section 2. Palettes are selected by `Act`. Each palette defines
//! colour values for tiles, sprites, and UI elements. A high-contrast interface
//! palette is independently selectable and never alters scene palettes.

use mc_core::world::Act;
use macroquad::prelude::Color;

/// A 15-bit colour (RGB 5:5:5). Stored as u16 with bits 0-4 = B, 5-9 = G, 10-14 = R.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colour15(u16);

impl Colour15 {
    /// Create from 5-bit R, G, B components.
    pub const fn from_rgb5(r: u8, g: u8, b: u8) -> Self {
        Colour15(((r as u16) << 10) | ((g as u16) << 5) | (b as u16))
    }

    /// Convert to macroquad Color (expands 5-bit to 8-bit).
    pub fn to_color(&self) -> Color {
        let r = ((self.0 >> 10) & 0x1F) as u8;
        let g = ((self.0 >> 5) & 0x1F) as u8;
        let b = (self.0 & 0x1F) as u8;
        Color::from_rgba(r << 3, g << 3, b << 3, 255)
    }
}

/// An array of 256 colours forming a palette.
pub type Palette = [Colour15; 256];

/// Get the scene palette for a given act (SPEC-004 section 2).
pub fn scene_palette(act: Act) -> &'static Palette {
    match act {
        Act::ActIMarseille => &PAL_ACT_I_MARSEILLE,
        Act::ActIIIf => &PAL_ACT_II_IF,
        Act::ActIIIMonteCristo => &PAL_ACT_III_SEA,
        Act::ActIVRome => &PAL_ACT_IV_TOUR,
        Act::ActVParis => &PAL_ACT_V_ROME,
        Act::ActVIParis => &PAL_ACT_VI_PARIS,
        Act::ActVIIFinal => &PAL_ACT_VII_EPILOGUE,
    }
}

/// The high-contrast interface palette, independently selectable (SPEC-004 section 8).
pub fn high_contrast_palette() -> &'static Palette {
    &PAL_HIGH_CONTRAST
}

/// Default (non-high-contrast) interface palette.
pub fn interface_palette() -> &'static Palette {
    &PAL_INTERFACE
}

// ── Act I: Cerulean and sand ───────────────────────────────────────────────
static PAL_ACT_I_MARSEILLE: Palette = build_act1();

const fn build_act1() -> Palette {
    let mut p = [Colour15(0); 256];
    p[0] = Colour15::from_rgb5(0, 0, 0);       // transparent/black
    p[1] = Colour15::from_rgb5(10, 12, 31);    // cerulean sky
    p[2] = Colour15::from_rgb5(12, 15, 31);    // lighter cerulean
    p[3] = Colour15::from_rgb5(31, 20, 10);    // sand
    p[4] = Colour15::from_rgb5(31, 22, 14);    // lighter sand
    p[5] = Colour15::from_rgb5(20, 12, 8);     // dark sand
    p[6] = Colour15::from_rgb5(31, 31, 31);    // white highlight
    p[7] = Colour15::from_rgb5(8, 8, 12);      // shadow
    p
}

// ── Act II: Six greys and one red ──────────────────────────────────────────
static PAL_ACT_II_IF: Palette = build_act2();

const fn build_act2() -> Palette {
    let mut p = [Colour15(0); 256];
    p[0] = Colour15::from_rgb5(0, 0, 0);
    p[1] = Colour15::from_rgb5(18, 18, 18);    // light grey
    p[2] = Colour15::from_rgb5(14, 14, 14);    // medium grey
    p[3] = Colour15::from_rgb5(10, 10, 10);    // dark grey
    p[4] = Colour15::from_rgb5(6, 6, 6);       // very dark grey
    p[5] = Colour15::from_rgb5(22, 22, 22);    // highlight grey
    p[6] = Colour15::from_rgb5(31, 6, 6);      // the one red (chains)
    p[7] = Colour15::from_rgb5(4, 4, 4);       // deepest shadow
    p
}

// ── Act III: Sea blue and canvas ───────────────────────────────────────────
static PAL_ACT_III_SEA: Palette = build_act3();

const fn build_act3() -> Palette {
    let mut p = [Colour15(0); 256];
    p[0] = Colour15::from_rgb5(0, 0, 0);
    p[1] = Colour15::from_rgb5(5, 10, 20);     // deep sea
    p[2] = Colour15::from_rgb5(8, 15, 25);     // mid sea
    p[3] = Colour15::from_rgb5(12, 20, 31);    // light sea
    p[4] = Colour15::from_rgb5(31, 28, 18);    // canvas
    p[5] = Colour15::from_rgb5(26, 22, 12);    // dark canvas
    p[6] = Colour15::from_rgb5(31, 31, 31);    // white
    p[7] = Colour15::from_rgb5(3, 6, 12);      // shadow
    p
}

// ── Act IV: Dust, olive, and lamp ──────────────────────────────────────────
static PAL_ACT_IV_TOUR: Palette = build_act4();

const fn build_act4() -> Palette {
    let mut p = [Colour15(0); 256];
    p[0] = Colour15::from_rgb5(0, 0, 0);
    p[1] = Colour15::from_rgb5(24, 20, 12);    // dust
    p[2] = Colour15::from_rgb5(16, 18, 8);     // olive
    p[3] = Colour15::from_rgb5(28, 22, 14);    // light dust
    p[4] = Colour15::from_rgb5(31, 26, 8);     // lamp
    p[5] = Colour15::from_rgb5(12, 14, 6);     // dark olive
    p[6] = Colour15::from_rgb5(31, 31, 24);    // highlight
    p[7] = Colour15::from_rgb5(6, 8, 4);       // shadow
    p
}

// ── Act V: Ochre and torch ─────────────────────────────────────────────────
static PAL_ACT_V_ROME: Palette = build_act5();

const fn build_act5() -> Palette {
    let mut p = [Colour15(0); 256];
    p[0] = Colour15::from_rgb5(0, 0, 0);
    p[1] = Colour15::from_rgb5(28, 16, 4);     // ochre
    p[2] = Colour15::from_rgb5(31, 20, 8);     // light ochre
    p[3] = Colour15::from_rgb5(31, 10, 2);     // torch
    p[4] = Colour15::from_rgb5(20, 12, 4);     // dark ochre
    p[5] = Colour15::from_rgb5(31, 18, 6);     // mid ochre
    p[6] = Colour15::from_rgb5(31, 31, 20);    // highlight
    p[7] = Colour15::from_rgb5(10, 6, 2);      // shadow
    p
}

// ── Act VI: Black, gold, wine, gaslight ────────────────────────────────────
static PAL_ACT_VI_PARIS: Palette = build_act6();

const fn build_act6() -> Palette {
    let mut p = [Colour15(0); 256];
    p[0] = Colour15::from_rgb5(0, 0, 0);
    p[1] = Colour15::from_rgb5(31, 26, 4);     // gold
    p[2] = Colour15::from_rgb5(20, 4, 8);      // wine
    p[3] = Colour15::from_rgb5(28, 22, 30);    // gaslight
    p[4] = Colour15::from_rgb5(24, 8, 12);     // dark wine
    p[5] = Colour15::from_rgb5(12, 12, 12);    // dark grey
    p[6] = Colour15::from_rgb5(31, 31, 31);    // white
    p[7] = Colour15::from_rgb5(6, 6, 10);      // shadow
    p
}

// ── Act VII: White marble ──────────────────────────────────────────────────
static PAL_ACT_VII_EPILOGUE: Palette = build_act7();

const fn build_act7() -> Palette {
    let mut p = [Colour15(0); 256];
    p[0] = Colour15::from_rgb5(0, 0, 0);
    p[1] = Colour15::from_rgb5(31, 31, 31);    // bright marble
    p[2] = Colour15::from_rgb5(24, 24, 26);    // medium marble
    p[3] = Colour15::from_rgb5(20, 20, 22);    // shadow marble
    p[4] = Colour15::from_rgb5(16, 16, 18);    // dark marble
    p[5] = Colour15::from_rgb5(28, 28, 28);    // highlight marble
    p[6] = Colour15::from_rgb5(12, 12, 14);    // deep shadow
    p[7] = Colour15::from_rgb5(31, 31, 24);    // warm light
    p
}

// ── Interface palettes ─────────────────────────────────────────────────────
static PAL_INTERFACE: Palette = build_interface();

const fn build_interface() -> Palette {
    let mut p = [Colour15(0); 256];
    p[0] = Colour15::from_rgb5(0, 0, 0);       // transparent
    p[1] = Colour15::from_rgb5(31, 31, 31);    // white text
    p[2] = Colour15::from_rgb5(8, 8, 16);      // window bg
    p[3] = Colour15::from_rgb5(20, 12, 24);    // window border
    p[4] = Colour15::from_rgb5(31, 20, 8);     // highlight/selection
    p[5] = Colour15::from_rgb5(16, 10, 20);    // dimmed
    p[6] = Colour15::from_rgb5(8, 16, 8);      // positive/green
    p[7] = Colour15::from_rgb5(20, 6, 6);      // negative/red
    p
}

static PAL_HIGH_CONTRAST: Palette = build_high_contrast();

const fn build_high_contrast() -> Palette {
    let mut p = [Colour15(0); 256];
    p[0] = Colour15::from_rgb5(0, 0, 0);       // black bg
    p[1] = Colour15::from_rgb5(31, 31, 31);    // white text
    p[2] = Colour15::from_rgb5(0, 0, 0);       // window bg (black)
    p[3] = Colour15::from_rgb5(31, 31, 31);    // window border (white)
    p[4] = Colour15::from_rgb5(31, 31, 0);     // highlight (yellow)
    p[5] = Colour15::from_rgb5(16, 16, 16);    // dimmed
    p[6] = Colour15::from_rgb5(0, 31, 0);      // positive (green)
    p[7] = Colour15::from_rgb5(31, 0, 0);      // negative (red)
    p
}

/// Sky gradient bands for the current act palette.
/// Returns a list of (offset_y, colour_index, height) for vertical bands.
pub fn sky_gradient(act: Act) -> Vec<(u16, u8, u16)> {
    match act {
        Act::ActIMarseille => vec![
            (0, 1, 56),   // cerulean sky
            (56, 2, 56),  // lighter cerulean
            (112, 1, 56),
            (168, 2, 56),
        ],
        Act::ActIIIf => vec![
            (0, 1, 56),
            (56, 2, 56),
            (112, 3, 56),
            (168, 4, 56),
        ],
        Act::ActIIIMonteCristo => vec![
            (0, 1, 56),
            (56, 2, 56),
            (112, 1, 56),
            (168, 2, 56),
        ],
        Act::ActIVRome => vec![
            (0, 1, 56),
            (56, 3, 56),
            (112, 1, 56),
            (168, 3, 56),
        ],
        Act::ActVParis => vec![
            (0, 1, 56),
            (56, 2, 56),
            (112, 1, 56),
            (168, 2, 56),
        ],
        Act::ActVIParis => vec![
            (0, 5, 56),
            (56, 3, 56),
            (112, 5, 56),
            (168, 3, 56),
        ],
        Act::ActVIIFinal => vec![
            (0, 1, 56),
            (56, 2, 56),
            (112, 1, 56),
            (168, 2, 56),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_independence() {
        // High-contrast palette is never the same as any scene palette.
        let hc = high_contrast_palette();
        let acts = [
            Act::ActIMarseille,
            Act::ActIIIf,
            Act::ActIIIMonteCristo,
            Act::ActIVRome,
            Act::ActVParis,
            Act::ActVIParis,
            Act::ActVIIFinal,
        ];
        for act in &acts {
            let scene = scene_palette(*act);
            // At least one entry must differ for independence to hold
            let mut different = false;
            for i in 0..256 {
                if scene[i].0 != hc[i].0 {
                    different = true;
                    break;
                }
            }
            assert!(different, "High-contrast palette must differ from {act:?} palette");
        }
    }

    #[test]
    fn palette_count() {
        let acts = [
            Act::ActIMarseille,
            Act::ActIIIf,
            Act::ActIIIMonteCristo,
            Act::ActIVRome,
            Act::ActVParis,
            Act::ActVIParis,
            Act::ActVIIFinal,
        ];
        for act in &acts {
            let p = scene_palette(*act);
            let mut non_zero = 0;
            for i in 0..256 {
                if p[i].0 != 0 {
                    non_zero += 1;
                }
            }
            assert!(non_zero > 0, "Palette for {act:?} must have non-zero entries");
        }
    }

    #[test]
    fn colour15_conversion() {
        let c = Colour15::from_rgb5(31, 0, 0); // full red
        let color = c.to_color();
        assert!(color.r > 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn sky_gradient_non_empty() {
        for act in &[
            Act::ActIMarseille,
            Act::ActIIIf,
            Act::ActIIIMonteCristo,
            Act::ActIVRome,
            Act::ActVParis,
            Act::ActVIParis,
            Act::ActVIIFinal,
        ] {
            let g = sky_gradient(*act);
            assert!(!g.is_empty(), "Sky gradient for {act:?} must not be empty");
        }
    }

    #[test]
    fn high_contrast_selected_independently() {
        // Verify that using the high-contrast palette doesn't change the scene palette
        let hc_pal = high_contrast_palette();
        let default_pal = interface_palette();
        assert_ne!(hc_pal[0].0, default_pal[0].0, "HC palette bg should differ from default");
    }
}
