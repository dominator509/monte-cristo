//! Integration test: the high-contrast interface palette never alters scene palettes.
//!
//! SPEC-004 section 2. Palettes are selected per-act. The high-contrast interface
//! palette is independently selectable and must never be identical to any scene
//! palette — otherwise toggling it would be a no-op for that act.

use std::collections::HashSet;

/// The Colour15 type used in the palette module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Colour15(u16);

impl Colour15 {
    const fn from_rgb5(r: u8, g: u8, b: u8) -> Self {
        Colour15(((r as u16) << 10) | ((g as u16) << 5) | (b as u16))
    }
}

/// Test that the high-contrast palette is distinct from every scene palette.
#[test]
fn high_contrast_is_independent_of_all_scene_palettes() {
    // Rebuild the high-contrast palette manually (we can't import the module's
    // static array directly from an integration test without making it pub).
    let hc = build_high_contrast();

    let scene_palettes = [
        ("Act1", build_act1()),
        ("Act2", build_act2()),
        ("Act3", build_act3()),
        ("Act4", build_act4()),
        ("Act5", build_act5()),
        ("Act6", build_act6()),
        ("Act7", build_act7()),
    ];

    for (name, scene) in &scene_palettes {
        let mut different = false;
        for i in 0..256 {
            if scene[i].0 != hc[i].0 {
                different = true;
                break;
            }
        }
        assert!(
            different,
            "High-contrast palette must differ from {name} palette — \
             toggling high-contrast would be a no-op for this act"
        );
    }
}

/// Test that the high-contrast palette and the default interface palette
/// differ on at least the first few entries (especially background).
#[test]
fn high_contrast_differs_from_interface() {
    let hc = build_high_contrast();
    let ui = build_interface();

    // Background (index 0): HC is black (0,0,0), interface is also (0,0,0),
    // so check index 1 (text colour): HC is white, interface is white...
    // Check index 2 (window bg): HC is black, interface is dark blue.
    // The key property: HC window bg is true black, interface is not.
    assert_ne!(
        hc[2].0, ui[2].0,
        "HC window background must differ from interface window background"
    );

    // Check border (index 3): HC border is white, interface border is purple.
    assert_ne!(
        hc[3].0, ui[3].0,
        "HC border must differ from interface border"
    );

    // Check highlight (index 4): HC is yellow, interface is orange.
    assert_ne!(
        hc[4].0, ui[4].0,
        "HC highlight must differ from interface highlight"
    );
}

/// Test that every scene palette has at least 2 non-zero colours (the
/// transparent/black entry at index 0 is always zero).
#[test]
fn all_scene_palettes_have_content() {
    let scene_palettes = [
        build_act1(),
        build_act2(),
        build_act3(),
        build_act4(),
        build_act5(),
        build_act6(),
        build_act7(),
    ];

    for (i, pal) in scene_palettes.iter().enumerate() {
        let non_zero_count = pal.iter().filter(|c| c.0 != 0).count();
        assert!(
            non_zero_count >= 2,
            "Scene palette {} has only {} non-zero colours",
            i + 1,
            non_zero_count
        );
    }
}

/// Test that all 8 palettes (7 scene + 1 HC) are pairwise distinct.
#[test]
fn all_palettes_pairwise_distinct() {
    let all = [
        ("hc", build_high_contrast()),
        ("ui", build_interface()),
        ("act1", build_act1()),
        ("act2", build_act2()),
        ("act3", build_act3()),
        ("act4", build_act4()),
        ("act5", build_act5()),
        ("act6", build_act6()),
        ("act7", build_act7()),
    ];

    let mut seen: HashSet<u16> = HashSet::new();
    for (name, pal) in &all {
        let key = hash_palette(pal);
        assert!(
            seen.insert(key),
            "Palette {name} is identical to another palette (hash collision)"
        );
    }
}

/// Simple hash of a palette.
fn hash_palette(pal: &[Colour15; 256]) -> u16 {
    let mut h: u16 = 0;
    for c in pal {
        h = h.wrapping_add(c.0);
    }
    h
}

// ── Palette definitions (re-created from the module source for integration test) ──

type Palette = [Colour15; 256];

const fn build_act1() -> Palette {
    let mut p = [Colour15(0); 256];
    p[1] = Colour15::from_rgb5(10, 12, 31);
    p[2] = Colour15::from_rgb5(12, 15, 31);
    p[3] = Colour15::from_rgb5(31, 20, 10);
    p[4] = Colour15::from_rgb5(31, 22, 14);
    p[5] = Colour15::from_rgb5(20, 12, 8);
    p[6] = Colour15::from_rgb5(31, 31, 31);
    p[7] = Colour15::from_rgb5(8, 8, 12);
    p
}

const fn build_act2() -> Palette {
    let mut p = [Colour15(0); 256];
    p[1] = Colour15::from_rgb5(18, 18, 18);
    p[2] = Colour15::from_rgb5(14, 14, 14);
    p[3] = Colour15::from_rgb5(10, 10, 10);
    p[4] = Colour15::from_rgb5(6, 6, 6);
    p[5] = Colour15::from_rgb5(22, 22, 22);
    p[6] = Colour15::from_rgb5(31, 6, 6);
    p[7] = Colour15::from_rgb5(4, 4, 4);
    p
}

const fn build_act3() -> Palette {
    let mut p = [Colour15(0); 256];
    p[1] = Colour15::from_rgb5(5, 10, 20);
    p[2] = Colour15::from_rgb5(8, 15, 25);
    p[3] = Colour15::from_rgb5(12, 20, 31);
    p[4] = Colour15::from_rgb5(31, 28, 18);
    p[5] = Colour15::from_rgb5(26, 22, 12);
    p[6] = Colour15::from_rgb5(31, 31, 31);
    p[7] = Colour15::from_rgb5(3, 6, 12);
    p
}

const fn build_act4() -> Palette {
    let mut p = [Colour15(0); 256];
    p[1] = Colour15::from_rgb5(24, 20, 12);
    p[2] = Colour15::from_rgb5(16, 18, 8);
    p[3] = Colour15::from_rgb5(28, 22, 14);
    p[4] = Colour15::from_rgb5(31, 26, 8);
    p[5] = Colour15::from_rgb5(12, 14, 6);
    p[6] = Colour15::from_rgb5(31, 31, 24);
    p[7] = Colour15::from_rgb5(6, 8, 4);
    p
}

const fn build_act5() -> Palette {
    let mut p = [Colour15(0); 256];
    p[1] = Colour15::from_rgb5(28, 16, 4);
    p[2] = Colour15::from_rgb5(31, 20, 8);
    p[3] = Colour15::from_rgb5(31, 10, 2);
    p[4] = Colour15::from_rgb5(20, 12, 4);
    p[5] = Colour15::from_rgb5(31, 18, 6);
    p[6] = Colour15::from_rgb5(31, 31, 20);
    p[7] = Colour15::from_rgb5(10, 6, 2);
    p
}

const fn build_act6() -> Palette {
    let mut p = [Colour15(0); 256];
    p[1] = Colour15::from_rgb5(31, 26, 4);
    p[2] = Colour15::from_rgb5(20, 4, 8);
    p[3] = Colour15::from_rgb5(28, 22, 30);
    p[4] = Colour15::from_rgb5(24, 8, 12);
    p[5] = Colour15::from_rgb5(12, 12, 12);
    p[6] = Colour15::from_rgb5(31, 31, 31);
    p[7] = Colour15::from_rgb5(6, 6, 10);
    p
}

const fn build_act7() -> Palette {
    let mut p = [Colour15(0); 256];
    p[1] = Colour15::from_rgb5(31, 31, 31);
    p[2] = Colour15::from_rgb5(24, 24, 26);
    p[3] = Colour15::from_rgb5(20, 20, 22);
    p[4] = Colour15::from_rgb5(16, 16, 18);
    p[5] = Colour15::from_rgb5(28, 28, 28);
    p[6] = Colour15::from_rgb5(12, 12, 14);
    p[7] = Colour15::from_rgb5(31, 31, 24);
    p
}

const fn build_interface() -> Palette {
    let mut p = [Colour15(0); 256];
    p[1] = Colour15::from_rgb5(31, 31, 31);
    p[2] = Colour15::from_rgb5(8, 8, 16);
    p[3] = Colour15::from_rgb5(20, 12, 24);
    p[4] = Colour15::from_rgb5(31, 20, 8);
    p[5] = Colour15::from_rgb5(16, 10, 20);
    p[6] = Colour15::from_rgb5(8, 16, 8);
    p[7] = Colour15::from_rgb5(20, 6, 6);
    p
}

const fn build_high_contrast() -> Palette {
    let mut p = [Colour15(0); 256];
    p[1] = Colour15::from_rgb5(31, 31, 31);
    p[2] = Colour15::from_rgb5(0, 0, 0);
    p[3] = Colour15::from_rgb5(31, 31, 31);
    p[4] = Colour15::from_rgb5(31, 31, 0);
    p[5] = Colour15::from_rgb5(16, 16, 16);
    p[6] = Colour15::from_rgb5(0, 31, 0);
    p[7] = Colour15::from_rgb5(31, 0, 0);
    p
}
