# Contributing to FGC

Thank you for your interest in contributing to FGC (Fax Garbage Collector)! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Development Setup](#development-setup)
- [Building the Project](#building-the-project)
- [Running Tests](#running-tests)
- [Code Style](#code-style)
- [Architecture Overview](#architecture-overview)
- [Key Components](#key-components)
- [Submitting Changes](#submitting-changes)
- [Code Review Process](#code-review-process)
- [Release Process](#release-process)

## Development Setup

### Prerequisites

Before you begin, ensure you have the following installed:

- **Rust**: Version 1.75 or later (install via [rustup](https://rustup.rs/))
- **cmake**: Required for building some dependencies
- **Git**: For version control
- **Linux or macOS**: Recommended for full feature support

#### Installing Prerequisites

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y cmake build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**macOS:**
```bash
brew install cmake
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Fedora/RHEL:**
```bash
sudo dnf install cmake gcc gcc-c++
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Cloning the Repository

```bash
git clone https://github.com/your-org/fgc.git
cd fgc
```

### Recommended IDE Setup

- **VS Code** with rust-analyzer extension
- **IntelliJ IDEA** with Rust plugin
- **Neovim** with rust-analyzer LSP

## Building the Project

### Basic Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Build with all features
cargo build --all-features
```

### Build Documentation

```bash
# Generate HTML documentation
cargo doc --open

# Generate documentation including private items
cargo doc --document-private-items --open
```

## Running Tests

### Test Commands

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests
cargo test --tests

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_allocate_tlab_memory_basic

# Run tests for specific module
cargo test --lib allocator

# Run tests with sanitizer (requires nightly)
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test

# Run thread sanitizer tests
RUSTFLAGS="-Zsanitizer=thread" cargo +nightly test
```

### Test Categories

FGC includes several types of tests:

| Test Type | Location | Purpose |
|-----------|----------|---------|
| Unit Tests | `src/**/*.rs` | Test individual functions and modules |
| Integration Tests | `tests/` | Test GC behavior end-to-end |
| Specification Tests | `tests/gc_spec_tests.rs` | Test GC correctness properties |
| Stress Tests | `tests/gc_stress.rs` | Test under heavy load |
| Benchmarks | `benches/` | Performance measurements |

### Running Benchmarks

```bash
# Run all benchmarks (requires nightly)
cargo +nightly bench

# Run specific benchmark
cargo +nightly bench gc_allocation

# Run benchmarks and save results
cargo +nightly bench -- --save-baseline main
```

## Code Style

### Rust API Guidelines

FGC follows the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

- **Naming**: Use clear, descriptive names
  - Types: `PascalCase` (e.g., `GarbageCollector`, `GcConfig`)
  - Functions: `snake_case` (e.g., `allocate`, `register_root`)
  - Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_HEAP_SIZE`)

- **Documentation**: All public items must have doc comments
  ```rust
  /// Brief description of what the function does.
  ///
  /// Longer description with more details if needed.
  /// Explain why, not just what.
  ///
  /// # Arguments
  ///
  /// * `size` - Size of allocation in bytes
  ///
  /// # Returns
  ///
  /// Address of allocated memory, or error if allocation fails.
  ///
  /// # Examples
  ///
  /// ```rust
  /// let addr = gc.allocate(64)?;
  /// ```
  ///
  /// # Errors
  ///
  /// Returns `FgcError::OutOfMemory` if heap is exhausted.
  pub fn allocate(&self, size: usize) -> Result<usize>;
  ```

- **Error Handling**: Use `Result<T, FgcError>` for recoverable errors
  ```rust
  pub fn allocate(&self, size: usize) -> Result<usize, FgcError> {
      if size > self.max_size {
          return Err(FgcError::OutOfMemory { requested: size, available: self.max_size });
      }
      // ...
  }
  ```

### Clippy

Run clippy before submitting:

```bash
# Run clippy with strict warnings
cargo clippy -- -D warnings

# Run clippy with all features
cargo clippy --all-features -- -D warnings

# Auto-fix clippy warnings
cargo clippy --fix
```

### Formatting

Use rustfmt for consistent formatting:

```bash
# Check formatting
cargo fmt --check

# Format code
cargo fmt
```

### Unsafe Code Guidelines

FGC uses `unsafe` internally. When writing unsafe code:

1. **Justify every unsafe block** with a comment explaining why it's safe
2. **Minimize unsafe scope** - keep unsafe blocks as small as possible
3. **Document safety invariants** in module-level docs
4. **Add tests** that exercise unsafe code paths

Example:
```rust
/// # Safety
///
/// This function is safe because:
/// 1. `ptr` is guaranteed to be aligned to 8 bytes
/// 2. `ptr` points to initialized memory of at least `size` bytes
/// 3. No other thread can modify this memory during the operation
pub unsafe fn read_object(ptr: *const u8, size: usize) -> Object {
    // Implementation
}
```

## Architecture Overview

FGC implements a concurrent mark-compact garbage collector with the following high-level architecture:

```
┌─────────────────────────────────────────────────────────────────┐
│                        FGC ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Runtime   │  │     GC      │  │   Config    │             │
│  │   (orch.)   │  │   (cycle)   │  │   (tuning)  │             │
│  └──────┬──────┘  └──────┬──────┘  └─────────────┘             │
│         │                │                                       │
│         │    ┌───────────┼───────────┐                          │
│         │    │           │           │                          │
│         ▼    ▼           ▼           ▼                          │
│  ┌─────────────────────────────────────────────────┐           │
│  │              Memory Subsystem                    │           │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐        │           │
│  │  │  Heap    │ │ Allocator│ │  Region  │        │           │
│  │  │          │ │          │ │          │        │           │
│  │  └──────────┘ └──────────┘ └──────────┘        │           │
│  └─────────────────────────────────────────────────┘           │
│         │                │                                       │
│    ┌────┴────┐      ┌────┴────┐                                │
│    ▼         ▼      ▼         ▼                                │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐                          │
│  │Barrier│ │Marker│ │Reloc.│ │ Stats│                          │
│  │(GC)   │ │(GC)  │ │(GC)  │ │      │                          │
│  └──────┘ └──────┘ └──────┘ └──────┘                          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Key Components

### Allocator (`src/allocator/`)

Responsible for high-speed, thread-safe object allocation.

**Key Files:**
- `mod.rs` - Main allocator orchestration
- `bump.rs` - Bump pointer allocator for small/medium objects
- `tlab.rs` - Thread-Local Allocation Buffer management
- `large.rs` - Large object allocator (> 4KB)
- `generational.rs` - Young/Old generation allocation

**Design Principles:**
- O(1) allocation for common case (TLAB hit)
- Thread-local where possible to reduce contention
- Size-class separation for efficiency

### Barrier (`src/barrier/`)

Implements colored pointers and load barriers for concurrent operations.

**Key Files:**
- `colored_ptr.rs` - ColoredPointer struct with metadata bits
- `load_barrier.rs` - Load barrier implementation
- `address_space.rs` - Multi-mapping virtual memory
- `read_barrier.rs` - Read barrier macros

**Design Principles:**
- Metadata in pointer bits (44-47) avoids object header overhead
- Fast path should be < 5ns
- Self-healing pointers during relocation

### Heap (`src/heap/`)

Region-based heap management with virtual memory operations.

**Key Files:**
- `mod.rs` - Main heap struct
- `region.rs` - Region lifecycle and state management
- `virtual_memory.rs` - Virtual memory reservation/commit
- `memory_mapping.rs` - Multi-mapping implementation
- `numa.rs` - NUMA-aware allocation

**Design Principles:**
- Reserve large address space upfront
- Commit physical memory on-demand
- Region-based for parallel collection

### Marker (`src/marker/`)

Concurrent marking system for identifying live objects.

**Key Files:**
- `mod.rs` - Main marker orchestrator
- `mark_queue.rs` - Work-stealing mark queue
- `bitmap.rs` - Mark bitmap per region
- `roots.rs` - Root scanning (stack, globals)
- `stack_scan.rs` - Stack unwinding for roots
- `object_scanner.rs` - Object reference scanning

**Design Principles:**
- Tri-color marking (White, Grey, Black)
- Concurrent with application
- Load barriers mark on access

### Relocate (`src/relocate/`)

Object relocation and compaction.

**Key Files:**
- `mod.rs` - Main relocator
- `forwarding.rs` - Forwarding tables
- `copy.rs` - Object copying
- `compaction.rs` - Region compaction

**Design Principles:**
- Concurrent copying with pointer healing
- Forwarding tables for indirection
- Self-healing pointers via load barriers

### Runtime (`src/runtime/`)

GC runtime integration with the application.

**Key Files:**
- `mod.rs` - Main runtime orchestrator
- `init.rs` - Initialization
- `safepoint.rs` - Safepoint management
- `finalizer.rs` - Object finalization

**Design Principles:**
- Minimal intrusion into application
- Polling-based safepoints
- Finalizer queue for cleanup

## Submitting Changes

### 1. Create an Issue

Before starting work, create or find an existing issue:

- Check existing issues to avoid duplicates
- Describe the problem or feature clearly
- Discuss approach if it's a significant change

### 2. Fork and Branch

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR-USERNAME/fgc.git
cd fgc

# Create a branch for your change
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-123-description
```

**Branch Naming:**
- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation changes
- `refactor/description` - Code refactoring
- `test/description` - Test additions

### 3. Implement Changes

- Follow the code style guidelines
- Add tests for new functionality
- Update documentation as needed
- Run all tests locally before pushing

### 4. Run Full Test Suite

```bash
# Ensure everything passes
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

### 5. Commit Changes

Use conventional commits format:

```bash
# Feature
git commit -m "feat: add work-stealing to mark queue"

# Bug fix
git commit -m "fix: prevent integer overflow in bump allocator"

# Documentation
git commit -m "docs: add examples to GarbageCollector API"

# Breaking change
git commit -m "feat!: change GcConfig field names for consistency"
```

### 6. Submit Pull Request

1. Push your branch: `git push origin feature/your-feature-name`
2. Open a PR on GitHub
3. Fill out the PR template:
   - Description of changes
   - Related issues
   - Testing performed
   - Breaking changes (if any)

## Code Review Process

### Review Timeline

- Initial review within 48 hours
- Address feedback within 7 days (or PR may be closed)
- Final merge after approval from maintainer

### Review Checklist

Reviewers will check:

- [ ] Code follows style guidelines
- [ ] All tests pass
- [ ] New code has tests
- [ ] Documentation is updated
- [ ] No unnecessary `unsafe`
- [ ] Error handling is appropriate
- [ ] Performance impact is acceptable

### Review Feedback

- Be respectful and constructive
- Explain the "why" behind suggestions
- Use suggestions feature for small changes
- Request re-review after addressing feedback

## Release Process

### Versioning

FGC follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Steps (Maintainers Only)

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with changes
3. Create release commit: `git commit -m "release: v0.1.0"`
4. Create and push tag: `git tag -a v0.1.0 -m "Release v0.1.0"`
5. Push to crates.io: `cargo publish`
6. Create GitHub release with changelog

### Release Schedule

- **Patch releases**: As needed for critical fixes
- **Minor releases**: Monthly or when significant features ready
- **Major releases**: As needed for breaking changes

## Getting Help

- **GitHub Issues**: For bugs and feature requests
- **Discussions**: For questions and general discussion
- **Discord**: Real-time chat (link in README)

## Code of Conduct

Please be respectful and inclusive in all interactions. We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## Thank You!

Your contributions make FGC better for everyone. Whether it's fixing a typo, adding tests, or implementing major features, every contribution is valued!
