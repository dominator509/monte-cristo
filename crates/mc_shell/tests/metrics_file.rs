//! EP-008 M2: Metrics file test — verifies metrics are written on clean exit.

#[test]
fn metrics_serializes_correctly() {
    let metrics = mc_shell::obs::SessionMetrics::new();
    let json = serde_json::to_string(&metrics).expect("serialize metrics");
    assert!(
        json.contains("\"session\""),
        "metrics json should contain session"
    );
    assert!(
        json.contains("\"build\""),
        "metrics json should contain build"
    );
    assert!(
        json.contains("\"start_time\""),
        "metrics json should contain start_time"
    );
    assert!(
        json.contains("\"end_time\""),
        "metrics json should contain end_time"
    );
    assert!(
        json.contains("\"ticks_elapsed\""),
        "metrics json should contain ticks_elapsed"
    );
}

#[test]
fn metrics_write_creates_file() {
    // Set MC_DATA_DIR to a temp path
    let dir = std::env::temp_dir().join("mc_test_metrics_file");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::env::set_var("MC_DATA_DIR", &dir);

    mc_shell::obs::record_tick();
    mc_shell::obs::record_command();
    mc_shell::obs::record_battle();

    let result = mc_shell::obs::write_metrics();
    assert!(result.is_ok(), "write_metrics should succeed: {:?}", result);

    // Check that a metrics file was created
    let entries = std::fs::read_dir(&dir.join("logs")).expect("read logs dir");
    let has_metrics = entries
        .filter_map(|e| e.ok())
        .any(|e| e.file_name().to_string_lossy().contains("metrics-"));
    assert!(has_metrics, "metrics file should exist in logs dir");

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::env::remove_var("MC_DATA_DIR");
}
