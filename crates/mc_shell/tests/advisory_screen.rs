//! EP-005 M8: Content advisory screen test.
//!
//! The advisory appears before the title screen on first run, is remembered,
//! and is re-readable from settings.

use mc_shell::config::ValidatedConfig;
use mc_shell::ui::advisory::ADVISORY_LINES;
use std::path::PathBuf;

fn tmp_cfg_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("mc_test_advisory_{}", name));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("create tmp dir");
    d
}

#[test]
fn advisory_shown_on_first_run() {
    let dir = tmp_cfg_dir("first_run");
    let config = ValidatedConfig::load_or_default(dir.clone());
    // Fresh config has advisory_acknowledged = false
    assert!(
        !config.advisory_acknowledged,
        "fresh config must not acknowledge advisory"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn advisory_acknowledged_persists() {
    let dir = tmp_cfg_dir("persists");
    let mut config = ValidatedConfig::load_or_default(dir.clone());
    config.advisory_acknowledged = true;
    config.save().expect("acknowledgement settings should save");

    let loaded = ValidatedConfig::load_or_default(dir.clone());
    assert!(
        loaded.advisory_acknowledged,
        "acknowledgement must persist after save"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn advisory_not_shown_after_acknowledge() {
    let dir = tmp_cfg_dir("not_shown");
    let mut config = ValidatedConfig::load_or_default(dir.clone());
    config.advisory_acknowledged = true;
    config.save().expect("acknowledgement settings should save");

    let loaded = ValidatedConfig::load_or_default(dir);
    assert!(
        loaded.advisory_acknowledged,
        "advisory must not be shown after acknowledgement"
    );
}

#[test]
fn advisory_copy_contains_the_required_warnings_and_acknowledgement_prompt() {
    let copy = ADVISORY_LINES.join(" ");
    assert!(copy.contains("suicide"));
    assert!(copy.contains("death of a child"));
    assert!(copy.contains("faithfully"));
    assert!(copy.contains("Z / ENTER / SPACE"));
}
