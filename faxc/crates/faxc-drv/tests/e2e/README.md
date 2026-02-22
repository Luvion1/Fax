# E2E Tests for Fax Compiler

This directory contains comprehensive End-to-End (E2E) tests for the Fax compiler (`faxc`). These tests verify the full compilation pipeline from Fax source code to executable.

## Test Structure

```
tests/e2e/
├── mod.rs              # Module declaration
├── compilation_tests.rs # Compilation pipeline tests
├── cli_tests.rs        # CLI interface tests
├── snapshot_tests.rs   # Snapshot/output comparison tests
├── README.md           # This documentation
├── fixtures/           # Test input files (.fax)
│   ├── hello_world.fax
│   ├── arithmetic.fax
│   ├── control_flow.fax
│   ├── functions.fax
│   ├── variables.fax
│   ├── loops.fax
│   ├── regression_qc002.fax
│   ├── invalid_syntax.fax
│   ├── sema_error.fax
│   ├── undeclared_var.fax
│   └── duplicate_fn.fax
└── snapshots/          # Expected output snapshots (*.snap)
```

## Running Tests

### Run All E2E Tests

```bash
# Run all tests in the faxc-drv package
cargo test --package faxc-drv

# Run only E2E tests (integration tests)
cargo test --package faxc-drv --test e2e

# Run all tests with output
cargo test --package faxc-drv -- --nocapture
```

### Run Specific Test Categories

```bash
# Run only compilation tests
cargo test --package faxc-drv compilation_tests -- --nocapture

# Run only CLI tests
cargo test --package faxc-drv cli_tests -- --nocapture

# Run only snapshot tests
cargo test --package faxc-drv snapshot_tests -- --nocapture
```

### Run Individual Tests

```bash
# Run a specific test by name
cargo test --package faxc-drv test_hello_world_compilation -- --nocapture

# Run tests matching a pattern
cargo test --package faxc-drv test_.*_compilation -- --nocapture
```

### Run with Verbose Output

```bash
# Show test execution details
cargo test --package faxc-drv -- --nocapture --test-threads=1

# Show compiler verbose output during tests
RUST_LOG=debug cargo test --package faxc-drv -- --nocapture
```

## Test Categories

### 1. Compilation Tests (`compilation_tests.rs`)

These tests verify that the compiler can successfully compile various Fax programs and properly handle errors.

| Test | Description | Expected Result |
|------|-------------|-----------------|
| `test_hello_world_compilation` | Simple "Hello, World!" program | Success |
| `test_arithmetic_operations` | Program with arithmetic operations | Success |
| `test_control_flow` | Program with if/else and while loops | Success |
| `test_functions_compilation` | Program with function definitions | Success |
| `test_variables_compilation` | Program with variable declarations | Success |
| `test_loops_compilation` | Program with while and for loops | Success |
| `test_regression_qc002` | Regression test for QC-002 fixes | Success |
| `test_invalid_syntax` | Program with invalid syntax | Failure (error) |
| `test_sema_errors` | Program with type mismatch | Failure (error) |
| `test_undeclared_variable_error` | Program using undeclared variable | Failure (error) |
| `test_duplicate_function_error` | Program with duplicate function | Failure (error) |
| `test_file_not_found_error` | Reference to non-existent file | Failure (error) |

### 2. CLI Tests (`cli_tests.rs`)

These tests verify the CLI interface of the Fax compiler.

| Test | Description | Expected Result |
|------|-------------|-----------------|
| `test_cli_help` | `--help` flag output | Contains usage info |
| `test_cli_version` | `--version` flag output | Contains version info |
| `test_cli_compile_file` | Compile file via CLI | Success |
| `test_cli_compile_output` | Compile with custom output path | Success |
| `test_cli_verbose` | `--verbose` flag output | Contains verbose info |

### 3. Snapshot Tests (`snapshot_tests.rs`)

These tests capture and compare compiler output to detect unintended changes.

| Test | Description | Snapshot File |
|------|-------------|---------------|
| `test_hello_world_snapshot` | Hello world verbose output | `hello_world_verbose.snap` |
| `test_arithmetic_snapshot` | Arithmetic verbose output | `arithmetic_verbose.snap` |
| `test_control_flow_snapshot` | Control flow verbose output | `control_flow_verbose.snap` |
| `test_invalid_syntax_snapshot` | Invalid syntax error output | `invalid_syntax_error.snap` |
| `test_sema_error_snapshot` | Semantic error output | `sema_error.snap` |
| `test_functions_snapshot` | Functions verbose output | `functions_verbose.snap` |
| `test_cli_help_snapshot` | CLI help output | `cli_help.snap` |
| `test_cli_version_snapshot` | CLI version output | `cli_version.snap` |

## Snapshot Testing

Snapshot tests capture the compiler's output and compare it against stored snapshots. This helps detect unintended changes in compiler behavior.

### How It Works

1. **First Run**: If no snapshot exists, the test creates one and passes with an info message.
2. **Subsequent Runs**: The test compares current output against the stored snapshot.
3. **CI Mode**: In CI (`CI=1`), mismatches cause test failure.
4. **Local Mode**: Locally, mismatches update the snapshot with a warning.

### Managing Snapshots

```bash
# Snapshots are stored in tests/e2e/snapshots/

# To reset all snapshots (delete and regenerate)
rm -rf tests/e2e/snapshots/*.snap
cargo test --package faxc-drv snapshot_tests

# To review snapshot changes
git diff tests/e2e/snapshots/
```

### CI Configuration

In CI environments, set the `CI` environment variable to enforce strict snapshot matching:

```yaml
# Example GitHub Actions
- name: Run E2E Tests
  run: cargo test --package faxc-drv --test e2e
  env:
    CI: true
```

## Writing New Tests

### Adding a Compilation Test

1. Create a new `.fax` fixture in `fixtures/`
2. Add a test function in `compilation_tests.rs`:

```rust
#[test]
fn test_my_feature() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("my_feature");
    let input_path = fixtures_dir().join("my_feature.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path);

    cmd.assert()
        .success()  // or .failure() for error tests
        .stderr(predicate::str::is_empty());

    assert!(output_path.exists(), "Output executable should exist");
}
```

### Adding a Snapshot Test

1. Add a test function in `snapshot_tests.rs`:

```rust
#[test]
fn test_my_feature_snapshot() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("my_feature");
    let input_path = fixtures_dir().join("my_feature.fax");

    let mut cmd = Command::new(faxc_bin());
    cmd.arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--verbose");

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert_snapshot("my_feature_verbose", &stderr);
    assert!(output.status.success(), "Compilation should succeed");
}
```

## Test Fixtures

### hello_world.fax
Simple "Hello, World!" program to test basic compilation.

### arithmetic.fax
Tests integer arithmetic operations (+, -, *, /).

### control_flow.fax
Tests if/else conditionals and while loops.

### functions.fax
Tests function definition, parameters, return values, and recursion.

### variables.fax
Tests variable declaration, types (Int, String, Bool), and reassignment.

### loops.fax
Tests while loops, for loops, nested loops, and break statements.

### regression_qc002.fax
Regression test for QC-002 fixes (comparison operators, equality).

### invalid_syntax.fax
Intentionally invalid syntax to test error handling.

### sema_error.fax
Type mismatch error to test semantic analysis.

### undeclared_var.fax
Undeclared variable usage to test scope resolution.

### duplicate_fn.fax
Duplicate function definition to test name resolution.

## Troubleshooting

### Tests Fail with "Binary Not Found"

Ensure the `faxc` binary is built:

```bash
cargo build --package faxc-drv --bin faxc
```

### Snapshot Tests Fail in CI

Check for unintended changes in compiler output:

```bash
# Run locally to see the diff
cargo test --package faxc-drv snapshot_tests -- --nocapture

# Review and commit updated snapshots if changes are expected
git diff tests/e2e/snapshots/
```

### Flaky Tests

If tests are flaky, run with single thread:

```bash
cargo test --package faxc-drv --test e2e -- --test-threads=1
```

## Coverage

To generate test coverage reports:

```bash
# Install cargo-tarpaulin if not installed
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --package faxc-drv --output-dir ./coverage --out Html

# Open coverage report
open ./coverage/tarpaulin-report.html
```

## Contributing

When adding new features to the compiler:

1. Add corresponding test fixtures in `fixtures/`
2. Add compilation tests in `compilation_tests.rs`
3. Add snapshot tests in `snapshot_tests.rs` (if output changes)
4. Run all tests to ensure nothing breaks
5. Update this README if adding new test categories

## Related Documentation

- [Compiler Driver](../../src/lib.rs) - Main driver implementation
- [QC Analysis](../../../../QC_ANALYSIS_2026-02-22.md) - Quality control analysis
- [SPEC.md](../../../../SPEC.md) - Language specification