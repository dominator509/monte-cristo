//! EP-005 M5: Text speed tests.
//!
//! Four speeds: Slow, Normal, Fast, Instant.
//! Hold-to-fast-forward never skips an unread line.

use mc_shell::ui::text::{text_speed_delay, TextSpeed};

#[test]
fn slow_is_slowest() {
    let s = text_speed_delay(TextSpeed::Slow);
    let n = text_speed_delay(TextSpeed::Normal);
    let f = text_speed_delay(TextSpeed::Fast);
    let i = text_speed_delay(TextSpeed::Instant);
    assert!(s > n, "Slow must be slower than Normal");
    assert!(n > f, "Normal must be slower than Fast");
    assert!(f > i, "Fast must be slower than Instant");
}

#[test]
fn instant_is_zero() {
    assert_eq!(text_speed_delay(TextSpeed::Instant), 0.0);
}

#[test]
fn all_speeds_positive() {
    for speed in &[TextSpeed::Slow, TextSpeed::Normal, TextSpeed::Fast] {
        let d = text_speed_delay(*speed);
        assert!(d > 0.0, "{speed:?} delay must be positive, got {d}");
    }
}

#[test]
fn no_combat_affordances_in_confidence() {
    // SPEC-004 §4, SPEC-010 §1, §2: Confidence scenes have no combat interface.
    // Verify by scanning the confidence module source.
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = std::fs::read_to_string(manifest_dir.join("src/ui/confidence.rs"))
        .expect("confidence.rs must exist");
    let lines: Vec<&str> = src.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let low = line.to_lowercase();
        for keyword in &["hp", "gauge", "atb", "turn_order"] {
            // Skip comments and strings that just mention the prohibition
            if low.contains(keyword) && !line.trim_start().starts_with("//") && !line.contains("no ") {
                panic!(
                    "confidence.rs:{} contains forbidden combat affordance '{}': {}",
                    i + 1,
                    keyword,
                    line
                );
            }
        }
    }
}
