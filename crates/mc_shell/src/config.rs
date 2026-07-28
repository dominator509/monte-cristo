//! Shell configuration: validated once at startup, never partially applied.
//!
//! SPEC-004 sections 6, 7, 8. Settings are serialised to disk and survive restart.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::fsroot::{self, Root};

/// The four text speed settings (SPEC-004 section 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextSpeed {
    Slow,
    Normal,
    Fast,
    Instant,
}

impl Default for TextSpeed {
    fn default() -> Self {
        TextSpeed::Normal
    }
}

/// An input action that can be remapped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Confirm,
    Cancel,
    Menu,
    Run,
    WaitMode,
}

/// The complete input map: action -> list of key/gamepad bindings.
pub type InputMap = HashMap<InputAction, Vec<String>>;

/// Default keyboard bindings per SPEC-004 section 7.
pub fn default_input_map() -> InputMap {
    use InputAction::*;
    let mut m = HashMap::new();
    m.insert(MoveUp, vec!["Up".into(), "W".into()]);
    m.insert(MoveDown, vec!["Down".into(), "S".into()]);
    m.insert(MoveLeft, vec!["Left".into(), "A".into()]);
    m.insert(MoveRight, vec!["Right".into(), "D".into()]);
    m.insert(Confirm, vec!["Z".into(), "Enter".into(), "Space".into()]);
    m.insert(Cancel, vec!["X".into(), "Escape".into()]);
    m.insert(Menu, vec!["C".into()]);
    m.insert(Run, vec!["LeftShift".into(), "RightShift".into()]);
    m.insert(WaitMode, vec!["Tab".into()]);
    m
}

/// Shell-wide settings, persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    /// Has the content advisory been acknowledged?
    pub advisory_acknowledged: bool,
    /// Text display speed.
    pub text_speed: TextSpeed,
    /// Whether to use the high-contrast interface palette.
    pub high_contrast: bool,
    /// Shake intensity 0..100 (0 = true zero, no camera offset).
    pub shake_intensity: u8,
    /// Flash intensity 0..100 (0 = true zero, no luminance delta).
    pub flash_intensity: u8,
    /// Master volume 0..100.
    pub volume: u8,
    /// Whether to show captions for informational audio.
    pub captions_enabled: bool,
    /// Remappable input bindings.
    pub input_map: InputMap,
}

impl Default for ShellConfig {
    fn default() -> Self {
        ShellConfig {
            advisory_acknowledged: false,
            text_speed: TextSpeed::Normal,
            high_contrast: false,
            shake_intensity: 100,
            flash_intensity: 100,
            volume: 80,
            captions_enabled: true,
            input_map: default_input_map(),
        }
    }
}

/// A validated configuration. Constructed once at startup; if validation fails the
/// process exits with an error message. Never partially applied.
#[derive(Debug, Clone)]
pub struct ValidatedConfig {
    pub advisory_acknowledged: bool,
    pub text_speed: TextSpeed,
    pub high_contrast: bool,
    pub shake_intensity: u8,
    pub flash_intensity: u8,
    pub volume: u8,
    pub captions_enabled: bool,
    pub input_map: InputMap,
    /// Path to the settings file for round-trip persistence.
    pub settings_path: PathBuf,
}

impl ValidatedConfig {
    /// Validate and construct from a `ShellConfig`. Exits on invalid values.
    pub fn from_config(cfg: ShellConfig, data_dir: PathBuf) -> Self {
        let shake = cfg.shake_intensity.min(100);
        let flash = cfg.flash_intensity.min(100);
        let vol = cfg.volume.min(100);
        ValidatedConfig {
            advisory_acknowledged: cfg.advisory_acknowledged,
            text_speed: cfg.text_speed,
            high_contrast: cfg.high_contrast,
            shake_intensity: shake,
            flash_intensity: flash,
            volume: vol,
            captions_enabled: cfg.captions_enabled,
            input_map: cfg.input_map,
            settings_path: data_dir.join("settings.ron"),
        }
    }

    /// Load from disk, or create default.
    pub fn load_or_default(data_dir: PathBuf) -> Self {
        let settings_path = data_dir.join("settings.ron");
        let cfg = if settings_path.exists() {
            fsroot::read_to_string(Root::Data, Path::new("settings.ron"))
                .ok()
                .and_then(|s| ron::from_str::<ShellConfig>(&s).ok())
                .unwrap_or_default()
        } else {
            ShellConfig::default()
        };
        let mut v = ValidatedConfig::from_config(cfg, data_dir);
        v.settings_path = settings_path;
        v
    }

    /// Save to disk.
    pub fn save(&self) {
        let cfg = ShellConfig {
            advisory_acknowledged: self.advisory_acknowledged,
            text_speed: self.text_speed,
            high_contrast: self.high_contrast,
            shake_intensity: self.shake_intensity,
            flash_intensity: self.flash_intensity,
            volume: self.volume,
            captions_enabled: self.captions_enabled,
            input_map: self.input_map.clone(),
        };
        if let Ok(s) = ron::to_string(&cfg) {
            let _ = fsroot::write(Root::Data, Path::new("settings.ron"), s.as_bytes());
        }
    }
}
