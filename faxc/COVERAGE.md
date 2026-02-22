# Fax Compiler - Code Coverage Report

## Overview

This document provides comprehensive code coverage analysis for the Fax Compiler workspace. The goal is to achieve and maintain **≥80% test coverage** across all crates.

## Coverage Targets

| Crate | Target | Status | Notes |
|-------|--------|--------|-------|
| faxc-util | ≥80% | ✅ | Core utilities - foundation types |
| faxc-lex | ≥80% | ✅ | Lexical analyzer |
| faxc-par | ≥80% | ✅ | Parser (recursive descent + Pratt) |
| faxc-sem | ≥80% | ✅ | Semantic analysis |
| faxc-mir | ≥80% | ✅ | Mid-level IR |
| faxc-lir | ≥80% | ✅ | Low-level IR |
| faxc-gen | ≥80% | ✅ | LLVM code generation |
| faxc-drv | ≥80% | ✅ | Driver and CLI |
| fgc | ≥80% | ✅ | Garbage collector |

## Test Structure

### Unit Tests

Unit tests are located within each source file using the `#[cfg(test)]` module pattern:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Test implementation
    }
}
```

### Integration Tests

Integration tests are located in the `tests/` directory of each crate:

```
faxc/
├── crates/
│   ├── faxc-drv/
│   │   ├── tests/
│   │   │   ├── integration_test.rs
│   │   │   ├── pipeline_integration.rs
│   │   │   └── edge_cases.rs
```

## Running Tests

### Run All Tests

```bash
cd faxc
cargo test --workspace
```

### Run Tests for Specific Crate

```bash
cargo test -p faxc-util
cargo test -p faxc-lex
cargo test -p faxc-par
```

### Run Tests with Output

```bash
cargo test --workspace -- --nocapture
```

### Run Specific Test

```bash
cargo test -p faxc-util test_def_id_generator
```

## Generating Coverage Reports

### Prerequisites

Install `cargo-llvm-cov`:

```bash
cargo install cargo-llvm-cov --version 0.6
```

### Using the Coverage Script

```bash
cd faxc

# Generate HTML report
./coverage-report.sh

# Generate LCOV report (for CI)
./coverage-report.sh --lcov

# Generate all reports
./coverage-report.sh --all

# Show help
./coverage-report.sh --help
```

### Manual Coverage Generation

```bash
# HTML Report
cargo llvm-cov --workspace --all-targets --html --output-dir target/llvm-cov/html

# LCOV Report
cargo llvm-cov --workspace --all-targets --lcov --output-path target/llvm-cov/lcov.info

# Cobertura Report
cargo llvm-cov --workspace --all-targets --cobertura --output-path target/llvm-cov/cobertura.xml

# Summary Only
cargo llvm-cov --workspace --all-targets --summary-only
```

### View HTML Report

Open the generated HTML report in your browser:

```bash
# Linux
xdg-open target/llvm-cov/html/index.html

# macOS
open target/llvm-cov/html/index.html

# Windows
start target/llvm-cov/html/index.html
```

## Coverage Thresholds

### Global Threshold
- **Minimum Coverage**: 80%

### Critical Crates Threshold
- **faxc-drv**: 85% (Driver - user-facing)
- **fgc**: 85% (Garbage Collector - critical infrastructure)
- **faxc-util**: 85% (Core utilities - foundation)

## Test Areas

### Unit Tests Coverage

| Area | Description | Status |
|------|-------------|--------|
| Error Types | Display implementations, error handling | ✅ |
| Utility Functions | Helper functions, algorithms | ✅ |
| Type Definitions | Structs, enums, traits | ✅ |
| Data Structures | IndexVec, Symbol table, etc. | ✅ |

### Integration Tests Coverage

| Area | Description | Status |
|------|-------------|--------|
| Full Pipeline | End-to-end compilation | ✅ |
| Error Scenarios | Error recovery, edge cases | ✅ |
| CLI Tests | Command-line interface | ✅ |
| API Tests | Public API contracts | ✅ |

## Test Best Practices

### 1. Test Naming Convention

```rust
#[test]
fn test_feature_under_test() { }

#[test]
fn test_edge_case_description() { }

#[test]
#[should_panic(expected = "error message")]
fn test_panic_condition() { }
```

### 2. Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Basic functionality tests
    #[test]
    fn test_basic_operation() { }

    // Edge case tests
    #[test]
    fn test_empty_input() { }

    #[test]
    fn test_boundary_values() { }

    // Error handling tests
    #[test]
    fn test_error_condition() { }
}
```

### 3. Test Coverage Goals

- **Lines**: ≥80%
- **Functions**: ≥80%
- **Regions**: ≥75%
- **Branches**: ≥75%

## CI Integration

Coverage is automatically checked in CI:

1. **On Push**: Coverage report generated and uploaded
2. **On PR**: Coverage diff calculated
3. **Threshold Check**: Fails if coverage drops below threshold

### GitHub Actions Workflow

See `.github/workflows/coverage.yml` for the complete workflow.

## Coverage Exclusions

Some code may be excluded from coverage:

```rust
#[cfg(test)]
mod tests { }

#[cfg(feature = "bench")]
mod benchmarks { }

#[doc(hidden)]
pub mod internal { }
```

## Troubleshooting

### Low Coverage in Specific File

1. Identify uncovered lines in HTML report
2. Add tests for uncovered branches/conditions
3. Consider if code is testable or needs refactoring

### False Negatives

Some code may appear uncovered but is actually tested:

- Macro-generated code
- Conditional compilation (`#[cfg(...)]`)
- Inline functions

### Coverage Not Generating

1. Ensure `cargo-llvm-cov` is installed
2. Check LLVM is available: `llvm-config --version`
3. Clean build: `cargo clean && cargo build`

## Reporting Issues

If you find uncovered code that should be tested:

1. Create an issue with the file and line numbers
2. Label with `A-testing` and `C-coverage`
3. Suggest test scenarios

## Related Documentation

- [Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [Rust Coverage Tools](https://rust-lang.github.io/rustup-components/)

## Version History

| Version | Date | Coverage | Notes |
|---------|------|----------|-------|
| 0.1.0 | 2026-02-21 | ≥80% | Initial coverage target achieved |

---

*Last Updated: 2026-02-21*