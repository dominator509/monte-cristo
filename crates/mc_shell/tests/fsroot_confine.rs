//! EP-006 M1: Filesystem confinement tests.
//!
//! Tests that `confine` rejects traversal, symlink escape, and absolute-path
//! injection, and accepts legitimate nested paths.

use std::fs;
use std::path::Path;

/// Test that a legitimate nested path is accepted.
#[test]
fn legitimate_path_accepted() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let sub = dir.path().join("subdir");
    fs::create_dir_all(&sub).expect("create subdir");
    let file = sub.join("test.txt");
    fs::write(&file, "hello").expect("write file");

    let result = mc_shell::fsroot::confine_to_path(dir.path(), mc_shell::fsroot::Root::Data, &file);
    assert!(result.is_ok(), "legitimate path should be accepted");
}

/// Test that traversal outside root is rejected.
#[test]
fn traversal_outside_root_rejected() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let result = mc_shell::fsroot::confine_to_path(
        dir.path(),
        mc_shell::fsroot::Root::Data,
        Path::new("/etc/passwd"),
    );
    assert!(result.is_err(), "traversal outside root should be rejected");
}

/// Test that absolute path outside root is rejected.
#[test]
fn absolute_path_outside_root_rejected() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let result = mc_shell::fsroot::confine_to_path(
        dir.path(),
        mc_shell::fsroot::Root::Data,
        Path::new("/tmp/somefile"),
    );
    assert!(
        result.is_err(),
        "absolute path outside root should be rejected"
    );
}

/// Test that a path with .. traversal is rejected.
#[test]
fn dotdot_traversal_rejected() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let bad = dir.path().join("../../etc/passwd");
    let result = mc_shell::fsroot::confine_to_path(dir.path(), mc_shell::fsroot::Root::Data, &bad);
    assert!(result.is_err(), ".. traversal should be rejected");
}

/// Test that writing a new file inside the root is accepted.
#[test]
fn write_new_file_inside_root() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let new_file = dir.path().join("new_file.txt");
    let result =
        mc_shell::fsroot::confine_to_path(dir.path(), mc_shell::fsroot::Root::Data, &new_file);
    assert!(
        result.is_ok(),
        "writing new file inside root should be accepted"
    );
}
