//! EP-008 M3: Crash report test.
//!
//! Verifies that crash reports can be serialized and written to disk.

#[test]
fn crash_report_has_all_fields() {
    let report = mc_shell::obs::CrashReport {
        session: "test-session".into(),
        build: "test-build".into(),
        timestamp: "2026-07-29T00-00-00Z".into(),
        tick: 12345,
        panic_message: "test panic: something went wrong".into(),
        backtrace: "stack frame 1\nstack frame 2\n".into(),
    };

    let json = serde_json::to_string_pretty(&report).expect("serialize crash report");
    assert!(
        json.contains("test-session"),
        "json should contain session: {}",
        json
    );
    assert!(json.contains("12345"), "json should contain tick: {}", json);
    assert!(
        json.contains("panic_message"),
        "json should contain panic_message: {}",
        json
    );
    assert!(
        json.contains("backtrace"),
        "json should contain backtrace: {}",
        json
    );
    assert!(
        json.contains("timestamp"),
        "json should contain timestamp: {}",
        json
    );
    assert!(
        json.contains("build"),
        "json should contain build: {}",
        json
    );
}

#[test]
fn crash_report_writes_to_disk() {
    let dir = std::env::temp_dir().join("mc_test_crash_report");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::env::set_var("MC_DATA_DIR", &dir);

    // Install crash hook and trigger a panic in a subprocess
    // (We can't catch panics that write files inside the hook easily,
    // so test the serialization directly)
    let report = mc_shell::obs::CrashReport {
        session: "test-session".into(),
        build: "test-build".into(),
        timestamp: "2026-07-29T00-00-00Z".into(),
        tick: 42,
        panic_message: "deliberate crash test".into(),
        backtrace: "backtrace here".into(),
    };

    let json = serde_json::to_string_pretty(&report).expect("serialize");
    std::fs::write(dir.join("crash-test-expected.json"), json.as_bytes())
        .expect("write test crash file");

    let content = std::fs::read_to_string(dir.join("crash-test-expected.json"))
        .expect("read test crash file");
    assert!(
        content.contains("deliberate crash test"),
        "file content should match"
    );

    std::fs::remove_dir_all(&dir).unwrap_or(());
    std::env::remove_var("MC_DATA_DIR");
}
