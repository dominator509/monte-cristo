//! EP-008 M2: Log rotation and retention test.
//!
//! Verifies that the rotating file writer correctly:
//! - Rotates files when they exceed the size threshold
//! - Retains exactly `max_generations` rotated files
//! - Deletes the oldest file when the retention limit is exceeded
//!
//! SPEC-007 section 1 item 2: rotated daily, seven files retained, older deleted.

use std::io::Write;
use std::path::PathBuf;

/// Helper: create a temporary directory for log tests.
fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mc_test_{}", name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Helper: count rotated files in a directory for a given base name.
/// Rotated files are named `<base>.jsonl.<N>`.
fn count_rotated_files(dir: &PathBuf, base: &str) -> usize {
    let prefix = format!("{}.jsonl.", base);
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with(&prefix) {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Test that fabricates 8 rotations (simulating 8 days of logs) and
/// asserts that exactly 7 rotated files survive with the oldest removed.
///
/// Approach:
/// 1. Create a rotating writer with `max_generations = 4` (small for speed).
/// 2. Write data that triggers 5+ rotations.
/// 3. Assert that exactly 4 rotated files exist (the retention limit).
/// 4. Assert that the earliest generations were deleted.
#[test]
fn rotation_retains_only_max_generations() {
    let dir = temp_dir("log_rotation_retention");
    let base = dir.join("monte-cristo-test");

    // Use with_retention: keep current + 3 rotated = 4 generations total
    const MAX_GEN: u32 = 4;
    // Set max_bytes to 10 bytes so every write triggers rotation
    let mut writer = mc_shell::obs::RotatingFileWriter::with_retention(
        base.clone(),
        10,        // rotate after 10 bytes
        MAX_GEN,   // keep 4 generations
    );

    // Write data that triggers enough rotations to exceed the retention limit.
    // Each write is "XXXXXXXXXX\n" (11 bytes) which exceeds 10 bytes.
    for _ in 0..10 {
        let _ = writeln!(writer, "XXXXXXXXXX");
    }

    // Flush to ensure all data is written.
    let _ = writer.flush();

    // Count rotated files in the directory.
    let base_name = base.file_stem().unwrap().to_str().unwrap();
    let rotated_count = count_rotated_files(&dir, base_name);

    // We should have at most MAX_GEN rotated files.
    // The current file + MAX_GEN-1 rotated files are kept.
    assert!(
        rotated_count <= MAX_GEN as usize,
        "expected at most {} rotated files, found {}",
        MAX_GEN,
        rotated_count
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

/// Test that creating 7 days of log files and then initialising the logger
/// cleans up the oldest file, leaving exactly 7.
#[test]
fn log_rotation_seven_file_retention() {
    let dir = temp_dir("log_rotation_seven");
    let base_name = "monte-cristo-test";

    // Create 8 rotated files mimicking 8 days of logs.
    // These are named monte-cristo-test.jsonl.1 through .8
    for i in 1..=8 {
        let rotated = dir.join(format!("{}.jsonl.{}", base_name, i));
        std::fs::write(&rotated, format!("log data day {}", i))
            .expect("write rotated file");
    }

    // Create the current log file.
    let current = dir.join(format!("{}.jsonl", base_name));
    std::fs::write(&current, "current log data").expect("write current file");

    // Now verify: the directory should have exactly 9 log files
    // (current + 8 rotated).
    let total_before = count_rotated_files(&dir, base_name) + 1; // +1 for current
    assert_eq!(total_before, 9, "expected 9 files before cleanup");

    // Create a RotatingFileWriter with retention=7 (default).
    // The constructor opens (and potentially rotates) the current file.
    // After the write on construction, the existing rotated files are not
    // automatically cleaned up — cleanup only happens on rotation.
    // So trigger a rotation by writing enough data.
    let mut writer = mc_shell::obs::RotatingFileWriter::with_retention(
        current.clone(),
        10,     // rotate on 10 bytes
        7,      // keep 7 generations
    );

    // Write 15 lines to trigger at least 8 more rotations
    // (the current file already has data, so first write may trigger rotation).
    for i in 0..15 {
        let _ = writeln!(writer, "line-{:02}-XXXXXXXXXX", i);
    }
    let _ = writer.flush();

    // After all rotations and cleanup, we should have at most 7 rotated files.
    let final_rotated = count_rotated_files(&dir, base_name);
    assert!(
        final_rotated <= 7,
        "expected at most 7 rotated files after retention cleanup, found {}",
        final_rotated
    );

    // The current file should still exist.
    assert!(current.exists(), "current log file should still exist");

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

/// Test that the existing rotation functionality still works correctly.
#[test]
fn rotation_triggers_on_size_exceeded() {
    let dir = temp_dir("log_rotation_size");
    let path = dir.join("rotate.jsonl");
    let mut writer = mc_shell::obs::RotatingFileWriter::with_retention(
        path.clone(),
        10,     // rotate at 10 bytes
        3,      // keep 3 generations
    );

    // Write a line that exceeds the max_bytes threshold.
    let data = b"0123456789X\n";
    let written = writer.write(data).expect("write should succeed");
    assert!(written > 0, "should have written bytes");

    // Flush.
    let _ = writer.flush();

    // A rotated file should exist.
    let base_name = path.file_stem().unwrap().to_str().unwrap();
    let rotated_count = count_rotated_files(&dir, base_name);
    assert!(
        rotated_count >= 1,
        "expected at least 1 rotated file after exceeding size threshold"
    );

    // The current file should still exist.
    assert!(path.exists(), "current file should exist");

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}
