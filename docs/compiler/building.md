# Building the Fax Compiler

Instructions for building the Fax compiler from source.

<!-- Source: faxc/Cargo.toml, README.md -->

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
| LLVM | 20.x | Code generation backend |

### Optional

| Tool | Version | Purpose |
|------|---------|---------|
| CMake | 3.10+ | Building LLVM (if needed) |
| Ninja | Latest | Fast builds |

### Installing Prerequisites

#### Ubuntu/Debian

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install LLVM 20 and system dependencies
sudo apt-get update
sudo apt-get install -y \
    git \
    cmake \
    ninja-build \
    clang \
    llvm-20 \
    llvm-20-dev \
    libpolly-20-dev \
    libzstd-dev \
    lld \
    libssl-dev \
    pkg-config

# Set LLVM path
export LLVM_SYS_200_PREFIX=/usr/lib/llvm-20
```

#### macOS

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install LLVM 20 and system dependencies
brew install \
    git \
    cmake \
    ninja \
    llvm@20 \
    openssl \
    pkg-config

# Set LLVM path
export LLVM_SYS_200_PREFIX=$(brew --prefix llvm@20)
```

#### Windows

```powershell
# Install Rust (using winget)
winget install Rustlang.Rustup

# Install LLVM 20
# Download from https://github.com/llvm/llvm-project/releases/tag/llvmorg-20.0.0

# Install Git
winget install Git.Git

# Set LLVM path (adjust based on installation location)
set LLVM_SYS_200_PREFIX=C:\Program Files\LLVM
```

---

## Quick Build

```bash
# Clone the repository
git clone https://github.com/Luvion1/Fax.git
cd Fax

# Set LLVM 20 path
export LLVM_SYS_200_PREFIX=/usr/lib/llvm-20

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

### Step 2: Verify Prerequisites

```bash
# Check Rust version (must be >= 1.75.0)
rustc --version

# Check LLVM version (must be 20.x)
llvm-config-20 --version

# If needed, update Rust
rustup update
```

### Step 3: Set LLVM 20 Path

```bash
# Ubuntu/Debian
export LLVM_SYS_200_PREFIX=/usr/lib/llvm-20

# macOS
export LLVM_SYS_200_PREFIX=$(brew --prefix llvm@20)

# Windows
set LLVM_SYS_200_PREFIX=C:\Program Files\LLVM
```

### Step 4: Build the Compiler

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

### Step 5: Run Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p faxc-lex

# Run tests with output
cargo test -- --nocapture
```

### Step 6: Verify the Build

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
| `LLVM_SYS_200_PREFIX` | Path to LLVM 20 installation | Auto-detect |

### Example Configuration

```bash
# Enable verbose output
export RUSTFLAGS="-v"

# Disable incremental compilation
export CARGO_INCREMENTAL=0

# Specify LLVM 20 path
export LLVM_SYS_200_PREFIX=/usr/lib/llvm-20
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

#### "LLVM 20 not found" or "LLVM_SYS_200_PREFIX not set"

```bash
# Ubuntu/Debian
sudo apt-get install llvm-20-dev libpolly-20-dev
export LLVM_SYS_200_PREFIX=/usr/lib/llvm-20

# macOS
brew install llvm@20
export LLVM_SYS_200_PREFIX=$(brew --prefix llvm@20)

# Windows
# Download LLVM 20 from https://github.com/llvm/llvm-project/releases/tag/llvmorg-20.0.0
set LLVM_SYS_200_PREFIX=C:\Program Files\LLVM
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

#### "Inkwell LLVM version mismatch"

The Fax compiler uses `inkwell` with the `llvm20-1` feature. Ensure your LLVM installation matches:

```bash
# Verify LLVM version
llvm-config-20 --version

# Should output: 20.x.x
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
