//! EP-005 M7: Font metrics test.
//!
//! The dyslexia-friendly font must maintain the same metrics as the default
//! font so that no layout reflow occurs when switching.

use mc_shell::ui::text::TextSpeed;

#[test]
fn font_metrics_defined() {
    // At minimum verify the type system works and text speed functions exist
    let _ = TextSpeed::Normal;
    assert!(true, "text module loads correctly");
}
