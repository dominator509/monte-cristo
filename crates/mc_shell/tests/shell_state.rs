use macroquad::prelude::KeyCode;
use mc_shell::app::{App, ScreenState, FIXED_DT};
use mc_shell::audio::{AudioState, CHANNELS, TRACKS};
use mc_shell::config::{
    default_input_map, InputAction, ShellConfig, TextSpeed as ConfigTextSpeed, ValidatedConfig,
};
use mc_shell::input::remap::{has_ambiguous_bindings, validate_map};
use mc_shell::input::{key_name_to_keycode, keycode_to_key_name};
use mc_shell::render::affine::{AffineLayer, AffineMatrix};
use mc_shell::render::sprite::{
    Sprite, BATTLE_SPRITE_H, BATTLE_SPRITE_W, EXPRESSION_FRAMES, FIELD_SPRITE_H, FIELD_SPRITE_W,
    PORTRAIT_H, PORTRAIT_W,
};
use mc_shell::render::tilemap::{TileLayer, Tilemap, TILES_X, TILES_Y};
use mc_shell::ui::text::{text_speed_delay, TextSpeed};
use std::path::PathBuf;
use std::process::Command;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "mc-shell-state-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temporary test directory should be creatable");
    dir
}

#[test]
fn headless_app_advances_the_authoritative_world_without_rendering() {
    let config = ValidatedConfig::from_config(ShellConfig::default(), temp_dir("app"));
    let mut app = App::new(0xC0FFEE, config, true);
    let initial_tick = app.world.tick;

    assert!(app.headless);
    assert!(app.render_target.is_none());
    assert!(!app.audio.enabled);
    assert_eq!(app.screen_state, ScreenState::Field);
    assert_eq!(app.tilemap.layer0.get_tile(0, 0), 0);
    assert_eq!(app.tilemap.layer0.get_tile(1, 0), 1);
    assert_eq!(app.tilemap.layer1.get_tile(0, 0), 2);
    assert_eq!(app.tilemap.layer1.get_tile(1, 0), 3);

    app.headless_update();

    assert_eq!(app.world.tick, initial_tick + 1);
    assert!(app.accum < FIXED_DT);
    assert!((0.0..1.0).contains(&app.alpha));
}

#[test]
fn validated_config_clamps_round_trips_and_falls_back_from_invalid_ron() {
    let dir = temp_dir("config");
    let config = ShellConfig {
        advisory_acknowledged: true,
        text_speed: ConfigTextSpeed::Fast,
        high_contrast: true,
        shake_intensity: 250,
        flash_intensity: 200,
        volume: 175,
        captions_enabled: false,
        input_map: default_input_map(),
    };
    let validated = ValidatedConfig::from_config(config, dir.clone());

    assert_eq!(validated.shake_intensity, 100);
    assert_eq!(validated.flash_intensity, 100);
    assert_eq!(validated.volume, 100);
    validated.save();

    let loaded = ValidatedConfig::load_or_default(dir.clone());
    assert!(loaded.advisory_acknowledged);
    assert_eq!(loaded.text_speed, ConfigTextSpeed::Fast);
    assert!(loaded.high_contrast);
    assert!(!loaded.captions_enabled);
    assert_eq!(loaded.settings_path, dir.join("settings.ron"));

    std::fs::write(dir.join("settings.ron"), "not valid ron")
        .expect("invalid settings fixture should be writable");
    let fallback = ValidatedConfig::load_or_default(dir);
    assert_eq!(fallback.text_speed, ConfigTextSpeed::Normal);
    assert_eq!(fallback.volume, 80);
}

#[test]
fn input_audio_and_render_state_enforce_their_public_contracts() {
    let mut input = default_input_map();
    assert!(!validate_map(&input));
    input.insert(InputAction::Confirm, vec!["Z".into(), "Enter".into()]);
    assert!(validate_map(&input));
    assert!(!has_ambiguous_bindings(&input));
    input.insert(InputAction::Cancel, vec!["Z".into()]);
    assert!(has_ambiguous_bindings(&input));

    let mut audio = AudioState::new(true);
    audio.update(1, mc_core::world::Act::ActIMarseille);
    assert!(audio.enabled);
    assert_eq!(audio.volume, 80);
    assert_eq!(audio.current_track, None);
    assert_eq!(CHANNELS, 8);
    assert_eq!(TRACKS, 34);
    assert!(AudioState::default().enabled);

    let identity = AffineMatrix::default();
    assert_eq!(
        (identity.a, identity.d, identity.tx, identity.ty),
        (1.0, 1.0, 0.0, 0.0)
    );
    let rotated = AffineMatrix::rotate_scale(std::f32::consts::FRAC_PI_2, 2.0, 4.0, 8.0);
    assert!(rotated.a.abs() < 0.0001);
    assert!((rotated.b + 2.0).abs() < 0.0001);
    assert!((rotated.c - 2.0).abs() < 0.0001);
    assert!(rotated.d.abs() < 0.0001);
    let layer = AffineLayer::default();
    assert!(!layer.visible);
    assert_eq!(layer.texture_id, None);

    let mut tiles = TileLayer::new(2, 2);
    tiles.set_tile(1, 1, 42);
    tiles.set_tile(2, 2, 99);
    assert_eq!(tiles.get_tile(1, 1), 42);
    assert_eq!(tiles.get_tile(2, 2), 0);
    let map = Tilemap::default();
    assert_eq!((map.layer0.width, map.layer0.height), (TILES_X, TILES_Y));

    let sprite = Sprite::new(12.0, 34.0);
    assert_eq!((sprite.x, sprite.y, sprite.frame), (12.0, 34.0, 0));
    assert!(sprite.visible);
    assert_eq!(
        (
            FIELD_SPRITE_W,
            FIELD_SPRITE_H,
            BATTLE_SPRITE_W,
            BATTLE_SPRITE_H,
            PORTRAIT_W,
            PORTRAIT_H,
            EXPRESSION_FRAMES,
        ),
        (24, 32, 48, 64, 64, 80, 8)
    );

    assert!(text_speed_delay(TextSpeed::Slow) > text_speed_delay(TextSpeed::Normal));
    assert!(text_speed_delay(TextSpeed::Normal) > text_speed_delay(TextSpeed::Fast));
    assert_eq!(text_speed_delay(TextSpeed::Instant), 0.0);
}

#[test]
fn every_supported_key_binding_round_trips_through_production_translation() {
    for name in [
        "Up",
        "Down",
        "Left",
        "Right",
        "W",
        "A",
        "S",
        "D",
        "Z",
        "X",
        "C",
        "Enter",
        "Escape",
        "Space",
        "Tab",
        "LeftShift",
        "RightShift",
    ] {
        let keycode = key_name_to_keycode(name).expect("default binding must be supported");
        assert_eq!(keycode_to_key_name(keycode), name);
    }
    assert_eq!(
        key_name_to_keycode("LShift").map(keycode_to_key_name),
        Some("LeftShift")
    );
    assert_eq!(
        key_name_to_keycode("RShift").map(keycode_to_key_name),
        Some("RightShift")
    );
    assert!(key_name_to_keycode("NotAKey").is_none());
    assert_eq!(keycode_to_key_name(KeyCode::F1), "Unknown");
}

#[test]
fn real_shell_entry_point_supports_version_and_headless_execution() {
    let binary = env!("CARGO_BIN_EXE_monte-cristo");
    let version = Command::new(binary)
        .arg("--version")
        .output()
        .expect("version command should launch");
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8_lossy(&version.stdout).trim(),
        format!("Monte Cristo v{}", env!("CARGO_PKG_VERSION"))
    );

    let dir = temp_dir("binary");
    let headless = Command::new(binary)
        .env("MC_HEADLESS", "1")
        .env("MC_DATA_DIR", &dir)
        .output()
        .expect("headless shell should launch");
    assert!(
        headless.status.success(),
        "headless stderr: {}",
        String::from_utf8_lossy(&headless.stderr)
    );
    assert!(dir.exists());

    let missing_content_dir = temp_dir("missing-content");
    let verify_content = Command::new(binary)
        .arg("--verify-content")
        .current_dir(&missing_content_dir)
        .output()
        .expect("content verification command should launch");
    assert!(!verify_content.status.success());
    assert!(
        String::from_utf8_lossy(&verify_content.stderr)
            .contains("Content verification failed for content.pack"),
        "verification stderr: {}",
        String::from_utf8_lossy(&verify_content.stderr)
    );
}
