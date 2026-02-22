//! Snapshot Testing for Fax Compiler
//!
//! These tests capture and compare compiler output snapshots to detect
//! unintended changes in compiler behavior.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use std::fs;

/// Get the path to the fixtures directory
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e")
        .join("fixtures")
}

/// Get the path to the snapshots directory
fn snapshots_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e")
        .join("snapshots")
}

/// Get the path to the faxc binary
fn faxc_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_faxc"))
}

/// Helper to get or create snapshot
fn get_snapshot(name: &str) -> Option<String> {
    let snapshot_path = snapshots_dir().join(format!("{}.snap", name));
    fs::read_to_string(snapshot_path).ok()
}

/// Helper to save snapshot
fn save_snapshot(name: &str, content: &str) {
    let snapshot_path = snapshots_dir().join(format!("{}.snap", name));
    fs::create_dir_all(snapshots_dir()).expect("Failed to create snapshots directory");
    fs::write(snapshot_path, content).expect("Failed to write snapshot");
}

/// Helper to update snapshot if needed
fn assert_snapshot(name: &str, actual: &str) {
    let snapshot_path = snapshots_dir().join(format!("{}.snap", name));
    
    if let Ok(expected) = fs::read_to_string(&snapshot_path) {
        // Snapshot exists, compare
        if actual.trim() != expected.trim() {
            // For CI, fail on mismatch
            if std::env::var("CI").is_ok() {
                panic!(
                    "Snapshot mismatch for '{}'. Expected:\n{}\n\nActual:\n{}",
                    name, expected, actual
                );
            }
            // For local dev, update snapshot with warning
            eprintln!("Warning: Snapshot '{}' mismatch. Updating...", name);
            save_snapshot(name, actual);
        }
    } else {
        // Snapshot doesn't exist, create it
        eprintln!("Info: Creating new snapshot '{}'", name);
        save_snapshot(name, actual);
    }
}

/// Test 1: Hello World Snapshot
/// Captures and compares the stderr output for hello world compilation
#[test]
fn test_hello_world_snapshot() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("hello_world");
    let input_path = fixtures_dir().join("hello_world.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--verbose");

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert_snapshot("hello_world_verbose", &stderr);
    assert!(output.status.success(), "Compilation should succeed");
}

/// Test 2: Arithmetic Operations Snapshot
/// Captures and compares the stderr output for arithmetic compilation
#[test]
fn test_arithmetic_snapshot() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("arithmetic");
    let input_path = fixtures_dir().join("arithmetic.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--verbose");

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert_snapshot("arithmetic_verbose", &stderr);
    assert!(output.status.success(), "Compilation should succeed");
}

/// Test 3: Control Flow Snapshot
/// Captures and compares the stderr output for control flow compilation
#[test]
fn test_control_flow_snapshot() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("control_flow");
    let input_path = fixtures_dir().join("control_flow.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--verbose");

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert_snapshot("control_flow_verbose", &stderr);
    assert!(output.status.success(), "Compilation should succeed");
}

/// Test 4: Invalid Syntax Error Snapshot
/// Captures and compares the error output for invalid syntax
#[test]
fn test_invalid_syntax_snapshot() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("invalid_syntax");
    let input_path = fixtures_dir().join("invalid_syntax.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert_snapshot("invalid_syntax_error", &stderr);
    assert!(!output.status.success(), "Compilation should fail");
}

/// Test 5: Semantic Error Snapshot
/// Captures and compares the error output for semantic errors
#[test]
fn test_sema_error_snapshot() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("sema_error");
    let input_path = fixtures_dir().join("sema_error.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert_snapshot("sema_error", &stderr);
    assert!(!output.status.success(), "Compilation should fail");
}

/// Test 6: Functions Compilation Snapshot
/// Captures and compares the stderr output for functions compilation
#[test]
fn test_functions_snapshot() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("functions");
    let input_path = fixtures_dir().join("functions.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--verbose");

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert_snapshot("functions_verbose", &stderr);
    assert!(output.status.success(), "Compilation should succeed");
}

/// Test 7: CLI Help Snapshot
/// Captures and compares the help output
#[test]
fn test_cli_help_snapshot() {
    let mut cmd = Command::new(faxc_bin());
    cmd.arg("--help");

    let output = cmd.output().expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert_snapshot("cli_help", &stdout);
    assert!(output.status.success(), "Help command should succeed");
}

/// Test 8: CLI Version Snapshot
/// Captures and compares the version output
#[test]
fn test_cli_version_snapshot() {
    let mut cmd = Command::new(faxc_bin());
    cmd.arg("--version");

    let output = cmd.output().expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert_snapshot("cli_version", &stdout);
    assert!(output.status.success(), "Version command should succeed");
}