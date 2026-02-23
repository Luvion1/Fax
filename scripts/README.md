# Fax Compiler Scripts

This directory contains development scripts for the Fax compiler project.

## Available Scripts

| Script | Description |
|--------|-------------|
| `install.sh` | Install LLVM 20 and Rust dependencies |
| `setup.sh` | Set up complete development environment |
| `build.sh` | Build the Fax compiler |
| `test.sh` | Run all tests |
| `run.sh` | Run example programs |
| `clean.sh` | Clean build artifacts |
| `fmt.sh` | Format code with rustfmt |
| `clippy.sh` | Run clippy linter |

## Quick Start

```bash
# 1. Install dependencies (first time only)
./scripts/install.sh

# 2. Set up development environment
./scripts/setup.sh

# 3. Build the compiler
./scripts/build.sh

# 4. Run tests
./scripts/test.sh

# 5. Run example programs
./scripts/run.sh --list
./scripts/run.sh 01
```

## Detailed Usage

### install.sh - Install Dependencies

Installs LLVM 20 and Rust toolchain.

```bash
./scripts/install.sh           # Install all dependencies
./scripts/install.sh --rust-only  # Only update Rust
./scripts/install.sh --check   # Verify installations
```

### setup.sh - Development Setup

Sets up complete development environment including:
- Rust toolchain verification
- LLVM verification
- Git hooks (pre-commit)
- Cargo configuration
- Editor configuration (.editorconfig)
- VS Code settings

```bash
./scripts/setup.sh             # Run full setup
./scripts/setup.sh --skip-deps # Skip dependency check
./scripts/setup.sh --force     # Force rebuild
```

### build.sh - Build Compiler

Builds the Fax compiler.

```bash
./scripts/build.sh             # Debug build
./scripts/build.sh --release   # Release build
./scripts/build.sh -j 4       # Parallel build
./scripts/build.sh --target x86_64-unknown-linux-gnu
./scripts/build.sh --all       # Build all crates
./scripts/build.sh --check     # Run cargo check
```

### run.sh - Run Examples

Compiles and runs Fax example programs.

```bash
./scripts/run.sh --list       # List all examples
./scripts/run.sh 01           # Run hello.fax
./scripts/run.sh --all         # Run all examples
./scripts/run.sh --build      # Build before running
./scripts/run.sh --release    # Use release build
./scripts/run.sh --emit asm hello  # Emit assembly
```

### test.sh - Run Tests

```bash
./scripts/test.sh              # Run all tests
```

### clean.sh - Clean Build Artifacts

```bash
./scripts/clean.sh             # Clean all
./scripts/clean.sh --target release  # Clean release only
./scripts/clean.sh --deps     # Clean with cargo cache
```

### fmt.sh - Format Code

```bash
./scripts/fmt.sh               # Format code
./scripts/fmt.sh --check      # Check formatting
```

### clippy.sh - Lint Code

```bash
./scripts/clippy.sh            # Run clippy
./scripts/clippy.sh --fix     # Fix warnings automatically
```

## Environment Variables

The following environment variables are used:

- `LLVM_SYS_20_PREFIX` - Path to LLVM 20 installation
- `CARGO_BUILD_JOBS` - Number of parallel build jobs
- `RUSTFLAGS` - Additional Rust compiler flags

## Troubleshooting

### LLVM not found

```bash
# Set LLVM path manually
export LLVM_SYS_20_PREFIX=/usr/lib/llvm-20

# Or install via install.sh
./scripts/install.sh
```

### Build fails

```bash
# Clean and rebuild
./scripts/clean.sh
./scripts/build.sh --release
```

### Tests fail

```bash
# Check formatting
./scripts/fmt.sh

# Run linter
./scripts/clippy.sh

# Run tests with verbose output
cargo test --workspace -- --nocapture
```
