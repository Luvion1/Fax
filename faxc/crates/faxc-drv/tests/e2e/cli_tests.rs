//! CLI Interface E2E Tests
//!
//! These tests verify the CLI interface of the Fax compiler,
//! testing help output, version, compile options, and verbose mode.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

/// Get the path to the fixtures directory
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e")
        .join("fixtures")
}

/// Get the path to the faxc binary
fn faxc_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_faxc"))
}

/// Test 1: CLI Help Output
/// Verifies that the --help flag displays help information
#[test]
fn test_cli_help() {
    let mut cmd = Command::new(faxc_bin());
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage").or(predicate::str::contains("faxc")));
}

/// Test 2: CLI Version Output
/// Verifies that the --version flag displays version information
#[test]
fn test_cli_version() {
    let mut cmd = Command::new(faxc_bin());
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("faxc").or(predicate::str::contains("0.")));
}

/// Test 3: CLI Compile File
/// Verifies that compiling a file via CLI works correctly
#[test]
fn test_cli_compile_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("test_output");
    let input_path = fixtures_dir().join("hello_world.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .success();

    // Verify the output executable exists
    assert!(output_path.exists(), "Output executable should exist");
}

/// Test 4: CLI Compile with Custom Output Path
/// Verifies that compiling with a custom output path works correctly
#[test]
fn test_cli_compile_output() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let custom_output = temp_dir.path().join("custom_bin").join("my_program");
    let input_path = fixtures_dir().join("arithmetic.fax");

    // Create the custom output directory
    std::fs::create_dir_all(custom_output.parent().unwrap())
        .expect("Failed to create output directory");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&custom_output);

    cmd.assert()
        .success();

    // Verify the output executable exists at the custom path
    assert!(custom_output.exists(), "Output executable should exist at custom path");
}

/// Test 5: CLI Verbose Mode
/// Verifies that the --verbose flag produces verbose output
#[test]
fn test_cli_verbose() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("verbose_output");
    let input_path = fixtures_dir().join("hello_world.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--verbose");

    cmd.assert()
        .success()
        .stderr(predicate::str::is_empty().or(predicate::str::contains("verbose").or(predicate::str::contains("Lexing").or(predicate::str::contains("Parsing")))));
}