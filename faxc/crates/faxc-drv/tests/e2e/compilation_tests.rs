//! Compilation Pipeline E2E Tests
//!
//! These tests verify the full compilation pipeline from Fax source code
//! to executable, testing various scenarios including successful compilation,
//! error handling, and semantic analysis.

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

/// Test 1: Hello World Compilation
/// Verifies that a simple "Hello, World!" program compiles successfully
#[test]
fn test_hello_world_compilation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("hello_world");
    let input_path = fixtures_dir().join("hello_world.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .success()
        .stderr(predicate::str::is_empty().or(predicate::str::contains("warning").not()));

    // Verify the output executable exists
    assert!(output_path.exists(), "Output executable should exist");
}

/// Test 2: Arithmetic Operations Compilation
/// Verifies that a program with arithmetic operations compiles successfully
#[test]
fn test_arithmetic_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("arithmetic");
    let input_path = fixtures_dir().join("arithmetic.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .success()
        .stderr(predicate::str::is_empty().or(predicate::str::contains("warning").not()));

    // Verify the output executable exists
    assert!(output_path.exists(), "Output executable should exist");
}

/// Test 3: Control Flow Compilation
/// Verifies that a program with if/else and while loops compiles successfully
#[test]
fn test_control_flow() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("control_flow");
    let input_path = fixtures_dir().join("control_flow.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .success()
        .stderr(predicate::str::is_empty().or(predicate::str::contains("warning").not()));

    // Verify the output executable exists
    assert!(output_path.exists(), "Output executable should exist");
}

/// Test 4: Invalid Syntax Error Handling
/// Verifies that the compiler properly handles invalid syntax
#[test]
fn test_invalid_syntax() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("invalid_syntax");
    let input_path = fixtures_dir().join("invalid_syntax.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));

    // Verify the output executable does NOT exist
    assert!(!output_path.exists(), "Output executable should not exist for invalid code");
}

/// Test 5: Semantic Error Handling
/// Verifies that the compiler properly handles semantic errors (type mismatch)
#[test]
fn test_sema_errors() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("sema_error");
    let input_path = fixtures_dir().join("sema_error.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));

    // Verify the output executable does NOT exist
    assert!(!output_path.exists(), "Output executable should not exist for semantic errors");
}

/// Test 6: Functions Compilation
/// Verifies that a program with function definitions and calls compiles successfully
#[test]
fn test_functions_compilation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("functions");
    let input_path = fixtures_dir().join("functions.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .success()
        .stderr(predicate::str::is_empty().or(predicate::str::contains("warning").not()));

    // Verify the output executable exists
    assert!(output_path.exists(), "Output executable should exist");
}

/// Test 7: Variables Compilation
/// Verifies that a program with various variable declarations compiles successfully
#[test]
fn test_variables_compilation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("variables");
    let input_path = fixtures_dir().join("variables.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .success()
        .stderr(predicate::str::is_empty().or(predicate::str::contains("warning").not()));

    // Verify the output executable exists
    assert!(output_path.exists(), "Output executable should exist");
}

/// Test 8: Loops Compilation
/// Verifies that a program with while and for loops compiles successfully
#[test]
fn test_loops_compilation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("loops");
    let input_path = fixtures_dir().join("loops.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .success()
        .stderr(predicate::str::is_empty().or(predicate::str::contains("warning").not()));

    // Verify the output executable exists
    assert!(output_path.exists(), "Output executable should exist");
}

/// Test 9: Regression Test QC-002
/// Verifies that previously fixed bugs remain fixed
#[test]
fn test_regression_qc002() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("regression_qc002");
    let input_path = fixtures_dir().join("regression_qc002.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .success()
        .stderr(predicate::str::is_empty().or(predicate::str::contains("warning").not()));

    // Verify the output executable exists
    assert!(output_path.exists(), "Output executable should exist for regression test");
}

/// Test 10: Undeclared Variable Error
/// Verifies that the compiler properly handles undeclared variable errors
#[test]
fn test_undeclared_variable_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("undeclared_var");
    let input_path = fixtures_dir().join("undeclared_var.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));

    // Verify the output executable does NOT exist
    assert!(!output_path.exists(), "Output executable should not exist for undeclared variable");
}

/// Test 11: Duplicate Function Error
/// Verifies that the compiler properly handles duplicate function definition errors
#[test]
fn test_duplicate_function_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("duplicate_fn");
    let input_path = fixtures_dir().join("duplicate_fn.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));

    // Verify the output executable does NOT exist
    assert!(!output_path.exists(), "Output executable should not exist for duplicate function");
}

/// Test 12: File Not Found Error
/// Verifies that the compiler properly handles missing input files
#[test]
fn test_file_not_found_error() {
    let mut cmd = Command::new(faxc_bin());
    cmd.arg("/nonexistent/path/to/file.fax");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")).or(predicate::str::contains("No such file")));
}