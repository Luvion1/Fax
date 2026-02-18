# Fax Compiler

[![Version](https://img.shields.io/badge/version-0.0.1--pre--alpha-orange.svg)](https://github.com/Luvion1/Fax/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![CI](https://img.shields.io/github/actions/workflow/status/username/faxc/ci.yml?branch=main)](https://github.com/Luvion1/Fax/actions)
[![Coverage](https://img.shields.io/github/actions/workflow/status/username/faxc/coverage.yml?branch=main&label=coverage)](https://github.com/Luvion1/Fax/actions)
[![Security](https://img.shields.io/github/actions/workflow/status/username/faxc/security-scan.yml?branch=main&label=security)](https://github.com/Luvion1/Fax/actions)

**A modern, functional-first programming language that compiles to LLVM IR.**

Fax combines the simplicity of imperative languages with the expressiveness of functional programming, featuring static typing with type inference, first-class functions, algebraic data types, and pattern matching—all compiling to high-performance native code via LLVM.

---

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage Examples](#usage-examples)
- [Documentation](#documentation)
- [Project Structure](#project-structure)
- [Contributing](#contributing)
- [Community](#community)
- [License](#license)

---

## Features

### Language Features

- **Functional-First Design** - First-class functions, immutability by default, higher-order functions
- **Static Typing with Inference** - Strong type system with intelligent type inference
- **Algebraic Data Types** - Enums and structs for expressive data modeling
- **Pattern Matching** - Powerful `match` expressions for control flow
- **Zero-Cost Abstractions** - Compile-time optimizations for native performance
- **Modern Syntax** - Clean, readable syntax inspired by Go and Rust

### Compiler Features

- **LLVM IR Code Generation** - Direct compilation to optimized LLVM IR
- **Cross-Platform Support** - Linux, macOS, and Windows
- **Fast Compilation** - Optimized compiler pipeline
- **Comprehensive Error Messages** - Clear, actionable diagnostics

### Garbage Collection (FGC)

- **Concurrent Mark-Compact GC** - Low-latency garbage collection
- **Generational Collection** - Optimized for object lifetimes
- **TLAB Allocation** - Thread-local allocation buffers
- **NUMA-Aware** - Optimized for multi-socket systems

---

## Quick Start

```bash
# Clone the repository
git clone https://github.com/Luvion1/Fax.git
cd faxc

# Build the compiler (debug mode)
./scripts/build.sh

# Build for release
./scripts/build.sh --release

# Run tests
./scripts/test.sh

# Compile and run a Fax program
./target/release/faxc examples/01_hello.fax
```

### Using Docker

```bash
# Build the Docker image
docker build -t faxc .

# Run the compiler in a container
docker run --rm -v $(pwd):/workspace faxc faxc /workspace/examples/01_hello.fax
```

---

## Installation

### Prerequisites

- **Rust 1.75 or later** ([install via rustup](https://rustup.rs))
- **LLVM 14+** (for code generation)
- **Git**

### Build from Source

```bash
# Clone the repository
git clone https://github.com/Luvion1/Fax.git
cd faxc

# Build in release mode
cd faxc
cargo build --release

# Add to PATH (optional)
export PATH="$PWD/target/release:$PATH"
```

### Verify Installation

```bash
# Check compiler version
faxc --version

# Run the test suite
./scripts/test.sh --release
```

---

## Usage Examples

### Hello World

```fax
fn main() {
    println("Hello, Fax!")
}
```

### Variables and Functions

```fax
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let x = 42              // immutable
    let mut y = 10          // mutable
    y = 20
    
    let result = add(x, y)
    println(result)         // prints 62
}
```

### Pattern Matching

```fax
enum Result {
    Ok(i32),
    Err(str),
}

fn main() {
    let value = Result::Ok(42)
    
    match value {
        Result::Ok(n) => println("Success: " + n),
        Result::Err(e) => println("Error: " + e),
    }
}
```

### Structs and Methods

```fax
struct Point {
    x: f64,
    y: f64,
}

fn distance(p: Point) -> f64 {
    (p.x * p.x + p.y * p.y).sqrt()
}

fn main() {
    let p = Point { x: 3.0, y: 4.0 }
    println(distance(p))    // prints 5.0
}
```

For more examples, see the [`examples/`](faxc/examples/) directory.

---

## Documentation

| Document | Description |
|----------|-------------|
| [Language Specification](SPEC.md) | Complete language grammar and semantics |
| [Architecture](faxc/docs/arch/ARCHITECT.md) | Compiler architecture overview |
| [API Documentation](faxc/crates/fgc/API.md) | FGC garbage collector API |
| [Contributing Guide](CONTRIBUTING.md) | How to contribute to Fax |
| [Release Notes](RELEASE.md) | Release process documentation |

---

## Project Structure

```
Fax/
├── faxc/                      # Main compiler crate
│   ├── crates/                # Compiler sub-crates
│   │   ├── faxc-lex/          # Lexer
│   │   ├── faxc-par/          # Parser
│   │   ├── faxc-util/         # Utilities
│   │   └── fgc/               # Garbage collector
│   ├── examples/              # Example programs
│   ├── scripts/               # Build and test scripts
│   └── docs/                  # Documentation
├── faxt/                      # Testing framework
├── .github/                   # GitHub workflows and templates
├── Dockerfile                 # Container build
└── SPEC.md                    # Language specification
```

---

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on:

- Code of Conduct
- Development setup
- Pull request process
- Coding standards
- Testing requirements

### Quick Contribution Guide

```bash
# Fork and clone
git clone https://github.com/Luvion1/Fax.git
cd faxc

# Create a branch
git checkout -b feature/my-feature

# Make changes and test
./scripts/test.sh

# Commit and push
git commit -m "feat: add my feature"
git push origin feature/my-feature
```

---

## Community

- **Issues**: [GitHub Issues](https://github.com/Luvion1/Fax/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Luvion1/Fax/discussions)
- **Security**: [Security Policy](SECURITY.md)

---

## License

Licensed under either of:

- **Apache License, Version 2.0** ([]())
- **MIT License** ([LICENSE](LICENSE))

at your option.

### Contribution License Agreement

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.

---

## Acknowledgments

Fax draws inspiration from:
- **Rust** - Type system and safety
- **Go** - Simplicity and readability
- **OCaml/Haskell** - Functional programming features
- **LLVM** - Compiler infrastructure

---

<div align="center">

**Fax Compiler** v0.0.1 pre-alpha

Made with ❤️ by the Fax Team

</div>
