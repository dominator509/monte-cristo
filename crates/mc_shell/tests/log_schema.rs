//! EP-008 M1: Log schema test — verifies that JSON logging produces
//! the required fields: ts, level, target, msg, session, build, tick.

use std::io::Write;

/// The log schema defines the exact set of fields and their types.
/// This test constructs a sample log event and verifies it can be
/// parsed with all required fields present.
#[test]
fn log_has_required_fields() {
    // Build a minimal JSON log line manually (as tracing-subscriber would)
    let entry = serde_json::json!({
        "ts": "2026-07-29T00:00:00Z",
        "level": "INFO",
        "target": "mc_shell::obs",
        "msg": "test log message",
        "session": "mc-test-session",
        "build": "dev",
        "tick": 42
    });

    let json = serde_json::to_string(&entry).expect("serialize log entry");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse log entry");

    assert!(parsed.get("ts").is_some(), "log entry must have ts");
    assert!(parsed.get("level").is_some(), "log entry must have level");
    assert!(parsed.get("target").is_some(), "log entry must have target");
    assert!(parsed.get("msg").is_some(), "log entry must have msg");
    assert!(
        parsed.get("session").is_some(),
        "log entry must have session"
    );
    assert!(parsed.get("build").is_some(), "log entry must have build");
    assert!(parsed.get("tick").is_some(), "log entry must have tick");
}

/// Verifies that a rotating file writer writes valid JSON lines.
#[test]
fn log_writer_produces_jsonl() {
    let dir = std::env::temp_dir().join("mc_test_log_schema");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let path = dir.join("test.jsonl");
    let max_bytes: u64 = 10 * 1024 * 1024;
    let mut writer =
        mc_shell::obs::RotatingFileWriter::new(path.clone(), max_bytes).expect("test log path");

    let entry = serde_json::json!({
        "ts": "2026-07-29T00:00:00Z",
        "level": "INFO",
        "msg": "test",
        "session": "s1",
        "build": "dev",
        "tick": 0
    });

    let line = serde_json::to_string(&entry).unwrap();
    writeln!(writer, "{}", line).expect("write log line");

    let content = std::fs::read_to_string(&path).expect("read log file");
    assert!(!content.is_empty(), "log file should have content");
    assert!(
        content.contains("\"ts\""),
        "log content should contain ts field"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn production_writer_emits_canonical_context_fields() {
    let dir = std::env::temp_dir().join("mc_test_log_schema_canonical");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let path = dir.join("test.jsonl");
    let mut writer = mc_shell::obs::CanonicalJsonWriter::new(path.clone(), 10 * 1024 * 1024)
        .expect("test log path");
    writeln!(
        writer,
        "{}",
        serde_json::json!({
            "timestamp": "2026-07-29T00:00:00Z",
            "level": "INFO",
            "target": "mc_shell::test",
            "message": "hello"
        })
    )
    .expect("write canonical log line");
    writer.flush().expect("flush canonical log line");

    let line = std::fs::read_to_string(path).expect("read canonical log file");
    let parsed: serde_json::Value = serde_json::from_str(line.trim()).expect("valid JSON line");
    assert!(parsed.get("ts").is_some());
    assert!(parsed.get("msg").is_some());
    assert!(parsed.get("session").is_some());
    assert!(parsed.get("build").is_some());
    assert!(parsed.get("tick").is_some());
    assert!(parsed.get("timestamp").is_none());
    assert!(parsed.get("message").is_none());

    let _ = std::fs::remove_dir_all(dir);
}
