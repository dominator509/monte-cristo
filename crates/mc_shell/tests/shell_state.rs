use macroquad::prelude::KeyCode;
use mc_core::command::Command as CoreCommand;
use mc_core::flags::FlagExpr;
use mc_core::ids::RegionId;
use mc_core::scene::{AuthoredNodeDefinition, AuthoredSceneCatalog, AuthoredSceneDefinition};
use mc_core::world::Act;
use mc_shell::app::{screen_state_after, App, ScreenState, FIXED_DT};
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
    assert!(
        !app.advisory_pending,
        "headless mode must not block on UI advisory"
    );
    assert_ne!(
        app.tilemap.layer0.get_tile(0, 0),
        app.tilemap.layer0.get_tile(2, 0),
        "the scene composition should not regress to a uniform startup pattern"
    );

    app.headless_update();

    assert_eq!(app.world.tick, initial_tick + 1);
    assert!(app.accum < FIXED_DT);
    assert!((0.0..1.0).contains(&app.alpha));
}

#[test]
fn authored_runtime_starts_at_the_available_arrest_scene() {
    let catalog = AuthoredSceneCatalog::from_definitions(vec![AuthoredSceneDefinition {
        id: "SCN_ARREST".into(),
        requires: FlagExpr::Always,
        nodes: vec![AuthoredNodeDefinition {
            id: "start".into(),
            text_key: "arrival.text".into(),
            choices: Vec::new(),
        }],
        on_exit: Vec::new(),
        terminal: false,
    }])
    .expect("arrival fixture should resolve");
    let config = ValidatedConfig::from_config(ShellConfig::default(), temp_dir("arrival"));
    let app = App::new_with_catalog(42, config, true, catalog).expect("app should start");

    assert_eq!(app.world.scene.map(|scene| scene.current.raw()), Some(14));
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
    validated.save().expect("validated settings should save");

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
fn settings_write_failure_returns_typed_error() {
    let dir = temp_dir("settings-error");
    let mut config = ValidatedConfig::load_or_default(dir.clone());
    config.settings_path = dir.join("settings.ron");
    std::fs::create_dir_all(&config.settings_path).expect("settings path fixture");

    let error = config
        .save()
        .expect_err("directory settings path must fail cleanly");
    assert!(error.to_string().contains("settings I/O failed"));
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
    assert_eq!(audio.current_track, Some(0));
    audio.update_with_scene(2, mc_core::world::Act::ActVIParis, Some(35));
    assert_eq!(
        audio.current_track,
        Some(1),
        "scene slots wrap to the authored soundtrack"
    );
    let mut muted = AudioState::new(false);
    muted.update(1, mc_core::world::Act::ActIMarseille);
    assert_eq!(muted.current_track, None);
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
fn scene_tilemaps_are_deterministic_and_region_specific() {
    let first = Tilemap::for_scene(Act::ActIMarseille, RegionId::R01_MARSEILLE);
    let repeat = Tilemap::for_scene(Act::ActIMarseille, RegionId::R01_MARSEILLE);
    let other = Tilemap::for_scene(Act::ActIMarseille, RegionId::R02_CHATEAU_DIF);

    assert_eq!(first.layer0.tiles, repeat.layer0.tiles);
    assert_eq!(first.layer1.tiles, repeat.layer1.tiles);
    assert_ne!(first.layer0.tiles, other.layer0.tiles);
}

#[test]
fn menu_commands_reach_and_leave_the_menu_overlay() {
    assert_eq!(
        screen_state_after(ScreenState::Field, &[CoreCommand::OpenMenu]),
        ScreenState::Menu
    );
    assert_eq!(
        screen_state_after(ScreenState::Menu, &[CoreCommand::CancelSelection]),
        ScreenState::Field
    );
    assert_eq!(
        screen_state_after(ScreenState::Menu, &[CoreCommand::CloseMenu]),
        ScreenState::Field
    );
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
    let log_dir = dir.join("logs");
    let log_file = std::fs::read_dir(&log_dir)
        .expect("headless shell should create a log directory")
        .filter_map(|entry| entry.ok())
        .find(|entry| entry.file_name().to_string_lossy().ends_with(".jsonl"))
        .expect("headless shell should create a JSONL log");
    let log = std::fs::read_to_string(log_file.path()).expect("read headless shell log");
    assert!(log.contains("logging initialised"));
    assert!(log.contains("metrics-"));
    assert!(!log.contains("C:\\Users\\"));
    assert!(!log.contains("/home/"));
    assert!(!log.contains("/Users/"));
    let metrics_file = std::fs::read_dir(&log_dir)
        .expect("read metrics directory")
        .filter_map(|entry| entry.ok())
        .find(|entry| entry.file_name().to_string_lossy().starts_with("metrics-"))
        .expect("headless shell should write clean-exit metrics");
    let metrics = std::fs::read_to_string(metrics_file.path()).expect("read headless metrics");
    assert!(metrics.contains("\"reference_machine\""));

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
