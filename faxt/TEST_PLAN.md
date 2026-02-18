# Comprehensive Test Plan for faxt CLI

## Overview

This test plan outlines a comprehensive testing strategy for the `faxt` CLI tool. The goal is to **find bugs** by testing requirements, not just validating current implementation.

## Test Philosophy

- **Test requirements, not implementation**: Tests should fail if implementation is wrong
- **Try to break the code**: Focus on edge cases and error conditions
- **Security-first mindset**: Actively test for path traversal, injection, etc.
- **Property-based testing**: Verify invariants hold for all inputs

---

## 1. Unit Tests

### 1.1 Path Sanitization (QC-005 Fix)

**File**: `tests/unit/path_sanitization.rs`

**Purpose**: Verify path traversal protection works correctly

**Test Scenarios**:
| Test | Input | Expected |
|------|-------|----------|
| Normal relative path | `./input` | Accepted |
| Parent directory traversal | `../etc/passwd` | Rejected |
| Deep traversal | `../../..` | Rejected |
| Hidden traversal | `./foo/../../../bar` | Rejected |
| Absolute path outside cwd | `/etc/passwd` | Rejected |
| Symlink to outside | symlink → `/etc` | Rejected |
| Unicode path | `输入/文件` | Accepted if within bounds |
| Very long path | 4096+ chars | Handled gracefully |

### 1.2 Config Loading

**File**: `tests/unit/config_loading.rs`

**Purpose**: Verify config loads from correct locations with correct precedence

**Test Scenarios**:
| Test | Scenario | Expected |
|------|----------|----------|
| Default config | No config file | Returns defaults |
| Current dir config | `./faxt.toml` exists | Loads from current dir |
| Home config | `~/.config/faxt/faxt.toml` | Loads from home |
| System config | `/etc/faxt/faxt.toml` | Loads from system |
| Precedence | Multiple configs exist | Current > Home > System |
| Invalid TOML | Malformed config file | Returns parse error |
| Missing required fields | Partial config | Uses defaults for missing |
| Quality bounds | quality = 0 | Should be rejected or clamped |
| Quality bounds | quality = 101 | Should be rejected or clamped |
| Jobs = 0 | jobs = 0 | Should default to >= 1 |

### 1.3 Error Message Formatting

**File**: `tests/unit/error_messages.rs`

**Purpose**: Verify error messages are clear, actionable, and don't leak sensitive info

**Test Scenarios**:
| Test | Scenario | Expected |
|------|----------|----------|
| Config error | Missing config | Clear message about what's missing |
| File operation error | Permission denied | Doesn't expose full path hierarchy |
| Validation error | Invalid input | Explains what's invalid and why |
| IO error chain | Nested IO errors | Preserves root cause |
| No secrets | Error with credentials | Credentials not in error message |

### 1.4 Argument Parsing Edge Cases

**File**: `tests/unit/arg_parsing.rs`

**Purpose**: Verify CLI argument parsing handles edge cases

**Test Scenarios**:
| Test | Input | Expected |
|------|-------|----------|
| Empty string arg | `--name ""` | Handled (empty name) |
| Very long arg | `--name "a".repeat(10000)` | Handled or rejected |
| Special chars | `--name "test\n\r\t"` | Escaped or rejected |
| Unicode args | `--name "日本語"` | Accepted |
| Negative numbers | `--quality -5` | Rejected by parser |
| Zero quality | `--quality 0` | Rejected (range 1-100) |
| Max quality | `--quality 100` | Accepted |
| Over max quality | `--quality 101` | Rejected by parser |
| Missing required | `convert` without input | Error about missing input |
| Unknown subcommand | `faxt unknown` | Helpful error message |

---

## 2. Integration Tests

### 2.1 Full Command Workflows

**File**: `tests/integration/workflows.rs`

**Purpose**: Verify complete workflows work end-to-end

**Test Scenarios**:
| Test | Workflow | Expected |
|------|----------|----------|
| Happy path | init → build → convert | All succeed |
| Init then build | init project, build | Build processes files |
| Build with clean | build --clean | Cleans before building |
| Convert chain | Multiple converts | All files converted |
| Config override | Config + CLI args | CLI args take precedence |

### 2.2 Config + CLI Precedence

**File**: `tests/integration/config_precedence.rs`

**Purpose**: Verify CLI args override config file values

**Test Scenarios**:
| Test | Config Value | CLI Value | Expected Result |
|------|--------------|-----------|-----------------|
| Quality | 50 | --quality 80 | 80 |
| Format | png | --format pdf | pdf |
| Jobs | 2 | --jobs 8 | 8 |
| Optimize | true | --no-optimize | false |
| Output | /config/out | --output /cli/out | /cli/out |
| Preserve meta | true | --preserve-metadata false | false |

### 2.3 File System Operations

**File**: `tests/integration/filesystem.rs`

**Purpose**: Verify real file operations work correctly

**Test Scenarios**:
| Test | Scenario | Expected |
|------|----------|----------|
| Create nested dirs | init with deep path | All dirs created |
| Copy large file | build with 100MB file | File copied correctly |
| Many files | build with 1000 files | All processed |
| Special filenames | Spaces, unicode | Handled correctly |
| Read-only files | build with read-only input | Error or skip |

---

## 3. Edge Case Tests

### 3.1 Empty Directories

**File**: `tests/edge_cases/empty_dirs.rs`

**Test Scenarios**:
| Test | Scenario | Expected |
|------|----------|----------|
| Build empty input | No files in input | Succeeds, 0 files processed |
| Convert empty list | No input files | Error: no input files |
| Init empty path | Valid empty dir | Succeeds |

### 3.2 Non-Existent Paths

**File**: `tests/edge_cases/nonexistent.rs`

**Test Scenarios**:
| Test | Scenario | Expected |
|------|----------|----------|
| Build non-existent input | `/no/such/dir` | Error: does not exist |
| Convert non-existent file | `/no/such/file` | Error: does not exist |
| Init non-existent parent | `/no/such/parent/project` | Creates parent dirs |
| Config non-existent | `--config /no/file.toml` | Error: not found |

### 3.3 Permission Denied

**File**: `tests/edge_cases/permissions.rs`

**Test Scenarios**:
| Test | Scenario | Expected |
|------|----------|----------|
| Read denied input | chmod 000 on input | Error: permission denied |
| Write denied output | chmod 555 on output | Error: permission denied |
| Execute in no-execute dir | Dir without +x | Error |
| Root-owned files | Files owned by root | Error or skip |

### 3.4 Very Long Paths

**File**: `tests/edge_cases/long_paths.rs`

**Test Scenarios**:
| Test | Scenario | Expected |
|------|----------|----------|
| Max path length | 255 char filename | Handled |
| Over max length | 256+ char filename | Error or truncated |
| Deep nesting | 100+ level directories | Handled or error |
| Path + filename | Combined > PATH_MAX | Error |

### 3.5 Unicode Filenames

**File**: `tests/edge_cases/unicode.rs`

**Test Scenarios**:
| Test | Filename | Expected |
|------|----------|----------|
| Chinese | `文件.fax` | Handled |
| Japanese | `ファイル.fax` | Handled |
| Emoji | `📁test.fax` | Handled |
| RTL | `ملف.fax` | Handled |
| Mixed | `test_文件_ファイル.fax` | Handled |
| Zero-width | `test.fax` (with ZWJ) | Handled |

### 3.6 Symlinks

**File**: `tests/edge_cases/symlinks.rs`

**Test Scenarios**:
| Test | Scenario | Expected |
|------|----------|----------|
| Symlink to file | input → file | Follows or rejects |
| Symlink to dir | input → dir | Follows or rejects |
| Circular symlink | loop → loop | Detected, error |
| Broken symlink | points to nothing | Error |
| Symlink escape | symlink → /etc | Rejected (security) |

### 3.7 Concurrent Operations

**File**: `tests/edge_cases/concurrent.rs`

**Test Scenarios**:
| Test | Scenario | Expected |
|------|----------|----------|
| Parallel builds | 2 builds same output | One succeeds or both fail cleanly |
| Race on output | Multiple writes | No corruption |
| Concurrent deletes | Clean while building | Handled gracefully |

---

## 4. Property-Based Tests

**File**: `src/commands/property_tests.rs`

### 4.1 Path Invariants

**Properties**:
- For any input path, sanitized output is within allowed directory
- `sanitize_path(p).starts_with(base_dir)` for all valid p

### 4.2 Config Value Bounds

**Properties**:
- Quality value always in range [1, 100] after validation
- Parallel jobs always >= 1
- Format always one of: pdf, png, jpeg, webp, tiff

### 4.3 Output Path Properties

**Properties**:
- Output path never overwrites input path without --force
- Output path has correct extension for format
- Output path is within allowed directory

---

## 5. Regression Tests

### 5.1 Infinite Recursion (Fixed Bug)

**File**: `tests/regression/infinite_recursion.rs`

**Test**: Verify directory traversal doesn't cause infinite loops

### 5.2 Path Traversal (Fixed Bug)

**File**: `tests/regression/path_traversal.rs`

**Test**: Verify `../` sequences are properly rejected

---

## 6. Security Tests

**File**: `tests/security/path_security.rs`

### 6.1 Path Traversal Attacks

| Attack Vector | Example | Expected |
|---------------|---------|----------|
| Basic traversal | `../secret` | Rejected |
| URL encoded | `..%2Fsecret` | Rejected |
| Double encoded | `..%252Fsecret` | Rejected |
| Unicode traversal | `..⁄secret` | Rejected |
| Null byte | `file.txt\0.jpg` | Rejected |
| Windows style | `..\..\secret` | Rejected |

### 6.2 Absolute Path Injection

| Attack | Example | Expected |
|--------|---------|----------|
| Absolute path | `/etc/passwd` | Rejected |
| UNC path | `\\server\share` | Rejected |
| Device path | `\\.\COM1` | Rejected |

---

## Test Execution

### Run All Tests
```bash
cargo test
```

### Run Specific Test Categories
```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Security tests
cargo test security

# Property-based tests
cargo test property
```

### Coverage Report
```bash
cargo tarpaulin --out Html
```

---

## Bug Reporting Template

When a test fails, document:

1. **Test Name**: Which test failed
2. **Expected**: What should have happened
3. **Actual**: What actually happened
4. **Steps to Reproduce**: Exact commands to reproduce
5. **Environment**: OS, Rust version, etc.
6. **Severity**: Critical/High/Medium/Low
7. **Security Impact**: Yes/No (if security-related)

---

## Test Coverage Goals

| Component | Target Coverage |
|-----------|-----------------|
| Commands (init, build, convert) | 90% |
| Config loading | 95% |
| Path sanitization | 100% |
| Error handling | 85% |
| Overall | 80%+ |

---

## Test Data

Test fixtures should be stored in `tests/fixtures/`:
- Sample config files (valid, invalid, edge cases)
- Sample input files (various formats, sizes)
- Directory structures for testing
