# Fax Compiler

<img src="docs/rubah-arktik.svg" width="100" alt="Fax Arctic Fox">

Fax is a modern systems programming language with static typing and garbage collection.

![MSRV](https://img.shields.io/badge/rustc-1.75+-blue.svg)
![LLVM](https://img.shields.io/badge/llvm-20.x-blue.svg)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)

## Minimum Supported Rust Version (MSRV)

This crate requires **Rust 1.75 or later**. The MSRV is tested in CI to ensure compatibility.

```bash
# Verify your Rust version
rustc --version  # Must be >= 1.75.0

# Install the minimum supported version
rustup install 1.75
rustup default 1.75
```

## LLVM Version Requirement

This crate requires **LLVM 20.x** for code generation. The compiler uses the `inkwell` crate with the `llvm20-1` feature.

### Setting LLVM Path

Before building, set the `LLVM_SYS_200_PREFIX` environment variable to point to your LLVM 20 installation:

```bash
# Ubuntu/Debian
export LLVM_SYS_200_PREFIX=/usr/lib/llvm-20

# macOS (Homebrew)
export LLVM_SYS_200_PREFIX=$(brew --prefix llvm@20)

# Windows (adjust path based on installation)
set LLVM_SYS_200_PREFIX=C:\Program Files\LLVM
```

## Features

- **Static Typing**: Type inference with comprehensive checking
- **Garbage Collection**: Low-latency concurrent GC (FGC)
- **Zero-Cost Abstractions**: Compile-time optimizations
- **Memory Safety**: No null pointers, no use-after-free
- **LLVM 20 Backend**: Modern optimization passes and code generation

## Quick Start

```bash
# Set LLVM 20 path
export LLVM_SYS_200_PREFIX=/usr/lib/llvm-20

# Build the compiler (debug mode)
cargo build

# Build for release (optimized)
cargo build --release

# Run tests
cargo test
```

## Build Scripts

```bash
# Debug build
./scripts/build.sh

# Release build
./scripts/build.sh --release

# Run tests
./scripts/test.sh

# Run all checks
./scripts/check.sh
```

## Example

```fax
fn main() {
    let message = "Hello, Fax!";
    print(message);
}
```

## Documentation

| Document | Description |
|----------|-------------|
| [Language Specification](../../SPEC.md) | Complete grammar reference |
| [Architecture Overview](docs/arch/overview.md) | Compiler architecture |
| [Compilation Pipeline](docs/arch/pipeline.md) | Compiler pipeline details |
| [Building Guide](docs/compiler/building.md) | Detailed build instructions |

## Project Structure

```
faxc/
├── crates/          # Compiler crates
│   ├── faxc-lex/    # Lexer
│   ├── faxc-par/    # Parser
│   ├── faxc-sem/    # Semantic analysis
│   ├── faxc-mir/    # Mid-level IR
│   ├── faxc-lir/    # Low-level IR
│   ├── faxc-gen/    # Code generation
│   ├── faxc-drv/    # Compiler driver
│   └── fgc/         # Garbage collector
├── examples/        # Example Fax programs
├── scripts/         # Build and test scripts
├── tests/           # Integration tests
└── docs/            # Documentation
```

## System Dependencies

### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install -y llvm-20-dev libpolly-20-dev libzstd-dev
```

### macOS

```bash
brew install llvm@20
```

### Windows

Download LLVM 20 from [llvm.org](https://llvm.org/releases/).

## License

MIT OR Apache-2.0
