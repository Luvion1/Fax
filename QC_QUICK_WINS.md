# Quick Wins - Fax Compiler QC Audit

**Audit ID:** QC-AUDIT-001  
**Date:** 2026-02-21  
**Purpose:** Issues that can be fixed in less than 1 hour each

---

## Overview

This document lists **quick wins** - issues identified during the QC audit that can be resolved quickly with minimal effort. Fixing these will immediately improve code quality and address some audit findings.

**Total Quick Wins:** 15  
**Estimated Total Time:** ~6 hours

---

## Quick Win #1: Fix std::process::exit(1) Violation

**Issue ID:** QC-001  
**Severity:** Critical  
**Time to Fix:** 5 minutes  
**Location:** `faxc/crates/faxc-drv/src/main.rs`

### Problem
The main.rs file uses `std::process::exit(1)` which violates the clippy configuration that disallows this macro.

### Current Code
```rust
fn main() {
    if let Err(e) = faxc_drv::main() {
        eprintln!("Error: {}", e);
        std::process::exit(1);  // ❌ Violates clippy config
    }
}
```

### Fix
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    faxc_drv::main()?;
    Ok(())
}
```

### Verification
```bash
cd faxc
cargo clippy -- -D warnings
```

---

## Quick Win #2: Create Missing scripts/ Directory

**Issue ID:** QC-029  
**Severity:** Medium  
**Time to Fix:** 10 minutes  
**Location:** Root directory

### Problem
README.md references `./scripts/build.sh` and `./scripts/test.sh` but these files don't exist.

### Fix

Create `scripts/build.sh`:
```bash
#!/bin/bash
# Fax Compiler Build Script

set -e

cd "$(dirname "$0")/.."

if [ "$1" = "--release" ]; then
    echo "Building Fax compiler in release mode..."
    cargo build --release --manifest-path faxc/Cargo.toml
else
    echo "Building Fax compiler in debug mode..."
    cargo build --manifest-path faxc/Cargo.toml
fi

echo "Build complete!"
```

Create `scripts/test.sh`:
```bash
#!/bin/bash
# Fax Compiler Test Script

set -e

cd "$(dirname "$0")/.."

echo "Running Fax compiler tests..."
cargo test --manifest-path faxc/Cargo.toml --workspace

echo "Tests complete!"
```

Make them executable:
```bash
chmod +x scripts/build.sh scripts/test.sh
```

### Verification
```bash
./scripts/build.sh
./scripts/test.sh
```

---

## Quick Win #3: Create Examples Directory Structure

**Issue ID:** QC-027, QC-030  
**Severity:** Medium  
**Time to Fix:** 15 minutes  
**Location:** Root directory

### Problem
Documentation references examples but no examples directory exists.

### Fix

Create directory structure:
```bash
mkdir -p examples
```

Create `examples/01_hello.fax`:
```fax
fn main() {
    println("Hello, Fax!")
}
```

Create `examples/02_variables.fax`:
```fax
fn main() {
    let x = 42
    let mut y = 10
    y = 20
    println("x = {}, y = {}", x, y)
}
```

Create `examples/03_functions.fax`:
```fax
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let result = add(3, 4)
    println("3 + 4 = {}", result)
}
```

Create `examples/README.md`:
```markdown
# Fax Examples

This directory contains example Fax programs demonstrating language features.

## Examples

| File | Description |
|------|-------------|
| 01_hello.fax | Hello World program |
| 02_variables.fax | Variable declarations and mutability |
| 03_functions.fax | Function definitions and calls |

## Running Examples

```bash
# Build the compiler
./scripts/build.sh --release

# Run an example
./target/release/faxc examples/01_hello.fax
```
```

### Verification
```bash
ls -la examples/
cat examples/01_hello.fax
```

---

## Quick Win #4: Add Module-Level Documentation

**Issue ID:** QC-025  
**Severity:** High  
**Time to Fix:** 20 minutes  
**Location:** `faxc/crates/faxc-par/src/lib.rs`

### Problem
The parser crate lacks comprehensive module-level documentation.

### Fix

Add to the top of `faxc/crates/faxc-par/src/lib.rs`:

```rust
//! # faxc-par - Parser for the Fax Programming Language
//!
//! This crate provides a recursive descent parser that transforms
//! tokens from the lexer into an Abstract Syntax Tree (AST).
//!
//! ## Features
//!
//! - Recursive descent parsing for statements and declarations
//! - Pratt parsing for expression precedence
//! - Panic-mode error recovery for robust error handling
//! - Comprehensive error diagnostics
//!
//! ## Usage
//!
//! ```rust,no_run
//! use faxc_util::Handler;
//! use faxc_lex::{Lexer, Token};
//! use faxc_par::Parser;
//!
//! let source = "fn main() { println(\"Hello\"); }";
//! let mut handler = Handler::new();
//! let mut lexer = Lexer::new(source, &mut handler);
//!
//! // Collect tokens
//! let tokens: Vec<Token> = std::iter::from_fn(|| Some(lexer.next_token()))
//!     .take_while(|t| *t != Token::Eof)
//!     .collect();
//!
//! // Parse
//! let mut parser = Parser::new(tokens, &mut handler);
//! let ast = parser.parse();
//! ```
//!
//! ## Error Recovery
//!
//! When encountering syntax errors, the parser uses panic-mode recovery
//! to skip to the next synchronization point and continue parsing.
```

### Verification
```bash
cd faxc
cargo doc --package faxc-par --no-deps
```

---

## Quick Win #5: Standardize Comment Language

**Issue ID:** QC-006  
**Severity:** Medium  
**Time to Fix:** 15 minutes  
**Location:** `faxc/crates/faxc-drv/src/lib.rs`

### Problem
Comments are mixed between Indonesian and English, which may confuse international contributors.

### Current Code
```rust
/// Configuration untuk compiler  // Indonesian
pub struct Config {
    // ...
}
```

### Fix
```rust
/// Compiler configuration
pub struct Config {
    // ...
}
```

### All Changes Needed in `faxc-drv/src/lib.rs`:
| Line | Current | Fix To |
|------|---------|--------|
| ~14 | `Configuration untuk compiler` | `Compiler configuration` |
| ~50+ | Various Indonesian comments | English equivalents |

### Verification
```bash
grep -r "untuk\|dengan\|atau" faxc/crates/faxc-drv/src/
# Should return no results
```

---

## Quick Win #6: Add Missing `# Returns` Documentation

**Issue ID:** QC-010  
**Severity:** Low  
**Time to Fix:** 20 minutes  
**Location:** Multiple files

### Problem
Some public functions lack `# Returns` sections in their documentation.

### Fix Pattern

For each function returning a value, add:

```rust
/// # Returns
///
/// A brief description of what is returned.
///
/// # Example
///
/// ```
/// // Example code
/// ```
```

### Files to Update:
- `faxc/crates/faxc-lex/src/lexer.rs` - ~5 functions
- `faxc/crates/faxc-par/src/lib.rs` - ~3 functions
- `faxc/crates/faxc-sem/src/lib.rs` - ~4 functions

### Verification
```bash
cd faxc
cargo doc --document-private-items 2>&1 | grep -i "missing"
```

---

## Quick Win #7: Document TODO Items

**Issue ID:** QC-008, QC-009  
**Severity:** Medium  
**Time to Fix:** 10 minutes  
**Location:** `fgc/src/`

### Problem
TODO comments without tracking issues.

### Fix

For each TODO, either:
1. Complete the implementation (if simple)
2. Create a GitHub issue and reference it

Example for `fgc/src/allocator/large.rs:131`:

```rust
// TODO(#123): Implement region splitting for better memory utilization
// Tracked in: https://github.com/fax-lang/faxc/issues/123
```

### Verification
```bash
grep -r "TODO" faxc/crates/fgc/src/ | grep -v "Tracked in"
```

---

## Quick Win #8: Add Test Fixtures

**Issue ID:** QC-012  
**Severity:** Low  
**Time to Fix:** 30 minutes  
**Location:** Test files

### Problem
Repeated test setup code across multiple test files.

### Fix

Create `faxc/crates/faxc-drv/tests/common.rs`:
```rust
//! Common test utilities and fixtures

use faxc_drv::{Config, Session, EmitType};
use faxc_util::Handler;

/// Create a default test configuration
pub fn test_config() -> Config {
    Config {
        input_files: Vec::new(),
        output_file: None,
        target: "x86_64-unknown-linux-gnu".to_string(),
        emit: EmitType::Lir,
        verbose: false,
        incremental: false,
    }
}

/// Create a test session with the given source code
pub fn test_session(source: &str) -> Session {
    let mut config = test_config();
    config.emit = EmitType::Lir;
    let mut session = Session::new(config);
    session.sources.add(
        std::path::PathBuf::from("test.fax"),
        source.to_string(),
    );
    session
}

/// Parse source code and return tokens
pub fn tokenize(source: &str) -> Vec<faxc_lex::Token> {
    let mut handler = Handler::new();
    let mut lexer = faxc_lex::Lexer::new(source, &mut handler);
    std::iter::from_fn(|| Some(lexer.next_token()))
        .take_while(|t| *t != faxc_lex::Token::Eof)
        .collect()
}
```

Update test files to use:
```rust
mod common;
use common::{test_config, test_session, tokenize};
```

### Verification
```bash
cd faxc
cargo test --package faxc-drv
```

---

## Quick Win #9: Add Safety Comments to Unsafe Code

**Issue ID:** QC-013  
**Severity:** High  
**Time to Fix:** 30 minutes  
**Location:** `fgc/src/`

### Problem
Unsafe blocks lack safety invariant documentation.

### Fix Pattern

```rust
/// # Safety
///
/// This function is safe because:
/// 1. The pointer is guaranteed to be valid and aligned
/// 2. The memory region is exclusively owned by this function
/// 3. No aliasing occurs during the operation
pub unsafe fn some_unsafe_function(ptr: *mut u8) {
    // SAFETY: ptr is validated by caller to be non-null and aligned
    *ptr = 0;
}
```

### Files to Update:
- `fgc/src/relocate/copy.rs` - Add safety comments to `aligned_copy`
- `fgc/src/barrier/fast_path.rs` - Document safety invariants
- `fgc/src/object/refmap.rs` - Document `from_raw` safety

### Verification
```bash
grep -A5 "pub unsafe" faxc/crates/fgc/src/ | grep -c "Safety"
```

---

## Quick Win #10: Update CHANGELOG

**Issue ID:** QC-026  
**Severity:** Medium  
**Time to Fix:** 10 minutes  
**Location:** `CHANGELOG.md`

### Problem
CHANGELOG only has initial release entry.

### Fix

Add to `CHANGELOG.md`:

```markdown
## [Unreleased]

### Added
- Initial pre-alpha release infrastructure
- Comprehensive CI/CD pipelines
- Security scanning with cargo-deny
- Docker support for containerized builds

### Changed
- Updated Dockerfile to use bookworm for security updates

### Fixed
- Various documentation improvements

### Security
- Configured cargo-deny for dependency security
- Implemented security scanning workflow
```

### Verification
```bash
cat CHANGELOG.md | head -30
```

---

## Quick Win #11: Add .gitignore for Common Patterns

**Issue ID:** QC-031 (related)  
**Severity:** Low  
**Time to Fix:** 5 minutes  
**Location:** Root and faxc/

### Problem
Ensure all build artifacts are properly ignored.

### Fix

Verify `.gitignore` includes:
```
# Build artifacts
/target/
**/target/

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log

# Coverage
*.profraw
*.profdata
coverage/

# Benchmarks
criterion/
```

### Verification
```bash
git status --ignored
```

---

## Quick Win #12: Add Benchmarks Placeholder

**Issue ID:** QC-032 (related)  
**Severity:** High  
**Time to Fix:** 15 minutes  
**Location:** `faxc/crates/fgc/benches/`

### Problem
No benchmark infrastructure.

### Fix

Create `faxc/crates/fgc/benches/allocation_bench.rs`:
```rust
//! GC Allocation Benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fgc::{GarbageCollector, GcConfig};

fn benchmark_allocation(c: &mut Criterion) {
    let config = GcConfig::default();
    let gc = GarbageCollector::new(config).unwrap();
    
    c.bench_function("allocate_64_bytes", |b| {
        b.iter(|| {
            let _addr = gc.allocate(64).unwrap();
            black_box(_addr);
        })
    });
}

criterion_group!(benches, benchmark_allocation);
criterion_main!(benches);
```

Add to `fgc/Cargo.toml`:
```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "allocation_bench"
harness = false
```

### Verification
```bash
cd faxc/crates/fgc
cargo bench --bench allocation_bench --no-run
```

---

## Quick Win #13: Add Clippy Exception for Tests

**Issue ID:** QC-002 (partial)  
**Severity:** Low  
**Time to Fix:** 5 minutes  
**Location:** `faxc/clippy.toml`

### Problem
Tests legitimately use unwrap/expect but clippy may warn.

### Fix

Already configured in `clippy.toml`:
```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
```

Verify this is working:
```bash
cd faxc
cargo clippy --tests -- -D warnings
```

If warnings appear, add `#[allow(clippy::unwrap_used)]` to test modules.

---

## Quick Win #14: Add Release Checklist

**Issue ID:** QC-026 (related)  
**Severity:** Low  
**Time to Fix:** 10 minutes  
**Location:** `RELEASE.md` or new `RELEASE_CHECKLIST.md`

### Fix

Create `RELEASE_CHECKLIST.md`:
```markdown
# Release Checklist

## Pre-Release
- [ ] All tests pass: `./scripts/test.sh`
- [ ] Clippy passes: `cargo clippy -- -D warnings`
- [ ] Documentation builds: `cargo doc --no-deps`
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Git tag created

## Release
- [ ] GitHub release created
- [ ] Docker image built and pushed
- [ ] Release notes published

## Post-Release
- [ ] Version bumped to next dev version
- [ ] Announcement made
```

---

## Quick Win #15: Add Issue Templates

**Issue ID:** QC-026 (related)  
**Severity:** Low  
**Time to Fix:** 15 minutes  
**Location:** `.github/ISSUE_TEMPLATE/`

### Fix

Create `.github/ISSUE_TEMPLATE/bug_report.md`:
```markdown
---
name: Bug Report
about: Report a bug
title: '[BUG] '
labels: bug
---

## Description
A clear description of the bug.

## To Reproduce
Steps to reproduce the behavior.

## Expected Behavior
What should happen.

## Environment
- OS: 
- Rust version:
- Fax version:
```

---

## Summary

| # | Quick Win | Time | Priority |
|---|-----------|------|----------|
| 1 | Fix std::process::exit | 5 min | Critical |
| 2 | Create scripts/ | 10 min | Medium |
| 3 | Create examples/ | 15 min | Medium |
| 4 | Add parser docs | 20 min | High |
| 5 | Standardize comments | 15 min | Medium |
| 6 | Add Returns docs | 20 min | Low |
| 7 | Document TODOs | 10 min | Medium |
| 8 | Add test fixtures | 30 min | Low |
| 9 | Safety comments | 30 min | High |
| 10 | Update CHANGELOG | 10 min | Medium |
| 11 | Verify .gitignore | 5 min | Low |
| 12 | Add benchmarks | 15 min | High |
| 13 | Clippy test config | 5 min | Low |
| 14 | Release checklist | 10 min | Low |
| 15 | Issue templates | 15 min | Low |
| **Total** | | **~6 hours** | |

---

## Execution Order

For maximum impact, execute in this order:

1. **Critical First:** Quick Win #1 (5 min)
2. **High Priority:** Quick Wins #4, #9, #12 (65 min)
3. **Medium Priority:** Quick Wins #2, #3, #5, #7, #10 (60 min)
4. **Low Priority:** Remaining (remaining time)

---

*Generated by SangPengawas QC Agent v2.0.0*