# Building the Fax Compiler

Instructions for building the Fax compiler from source.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Build](#quick-build)
3. [Detailed Build Steps](#detailed-build-steps)
4. [Build Options](#build-options)
5. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Compiler toolchain |
| Git | Latest | Version control |

### Optional

| Tool | Version | Purpose |
|------|---------|---------|
| LLVM | 14+ | Code generation backend |
| CMake | 3.10+ | Building LLVM (if needed) |
| Ninja | Latest | Fast builds |

### Installing Prerequisites

#### Ubuntu/Debian

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install system dependencies
sudo apt-get update
sudo apt-get install -y \
    git \
    cmake \
    ninja-build \
    clang \
    llvm \
    lld \
    libssl-dev \
    pkg-config
```

#### macOS

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install system dependencies
brew install \
    git \
    cmake \
    ninja \
    llvm \
    openssl \
    pkg-config
```

#### Windows

```powershell
# Install Rust (using winget)
winget install Rustlang.Rustup

# Install LLVM
winget install LLVM.LLVM

# Install Git
winget install Git.Git
```

---

## Quick Build

```bash
# Clone the repository
git clone https://github.com/Luvion1/Fax.git
cd Fax

# Build in debug mode
cd faxc
cargo build

# Build in release mode (optimized)
cargo build --release

# Run tests
cargo test

# Install (optional)
cargo install --path .
```

---

## Detailed Build Steps

### Step 1: Clone the Repository

```bash
git clone https://github.com/Luvion1/Fax.git
cd Fax
```

### Step 2: Verify Rust Version

```bash
# Check Rust version (must be >= 1.75.0)
rustc --version

# If needed, update Rust
rustup update
```

### Step 3: Build the Compiler

#### Debug Build (for development)

```bash
cd faxc
cargo build
```

This creates the binary at `target/debug/faxc`.

#### Release Build (optimized)

```bash
cd faxc
cargo build --release
```

This creates the binary at `target/release/faxc`.

### Step 4: Run Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p faxc-lex

# Run tests with output
cargo test -- --nocapture
```

### Step 5: Verify the Build

```bash
# Check compiler version
./target/release/faxc --version

# Compile a test program
echo 'fn main() { println("Build successful!") }' > test.fax
./target/release/faxc test.fax
./test
```

---

## Build Options

### Cargo Profiles

The project defines custom profiles in `Cargo.toml`:

```toml
[profile.release]
opt-level = 3      # Maximum optimization
lto = true         # Link-time optimization
codegen-units = 1  # Single codegen unit for better optimization

[profile.dev]
opt-level = 1      # Some optimization for faster debugging
```

### Build Features

```bash
# Build with all features
cargo build --all-features

# Build without default features
cargo build --no-default-features

# Build specific crate
cargo build -p faxc-lex
```

### Cross-Compilation

```bash
# Add target
rustup target add x86_64-unknown-linux-gnu

# Build for target
cargo build --target x86_64-unknown-linux-gnu --release
```

### Build Scripts

The project includes helper scripts:

```bash
# Build (debug)
./scripts/build.sh

# Build (release)
./scripts/build.sh --release

# Run all tests
./scripts/test.sh

# Run all checks
./scripts/check.sh
```

---

## Build Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUSTFLAGS` | Additional flags for rustc | - |
| `CARGO_INCREMENTAL` | Enable incremental compilation | 1 |
| `LLVM_CONFIG_PATH` | Path to llvm-config | Auto-detect |

### Example Configuration

```bash
# Enable verbose output
export RUSTFLAGS="-v"

# Disable incremental compilation
export CARGO_INCREMENTAL=0

# Specify LLVM path
export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14
```

---

## Troubleshooting

### Common Issues

#### "Rust version too old"

```bash
# Update Rust
rustup update

# Verify version
rustc --version  # Must be >= 1.75.0
```

#### "LLVM not found"

```bash
# Ubuntu/Debian
sudo apt-get install llvm-dev

# macOS
brew install llvm
export PATH="$(brew --prefix llvm)/bin:$PATH"

# Windows
# Ensure LLVM is in PATH
```

#### "Build fails with linker errors"

```bash
# Install build essentials
sudo apt-get install build-essential

# macOS
xcode-select --install
```

#### "Tests fail"

```bash
# Clean and rebuild
cargo clean
cargo build

# Run tests again
cargo test
```

### Getting Help

- Check the [FAQ](faq.md)
- Search [existing issues](https://github.com/Luvion1/Fax/issues)
- Start a [discussion](https://github.com/Luvion1/Fax/discussions)

---

## Performance Tips

### Faster Builds

```bash
# Use sccache for caching
cargo install sccache
export RUSTC_WRAPPER=sccache

# Use mold linker (faster than lld)
cargo install mold
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
```

### Smaller Binaries

```bash
# Strip symbols
strip target/release/faxc

# Build with size optimization
cargo build --release --target x86_64-unknown-linux-musl
```

---

<div align="center">

**Happy building!** 🔨

</div>
