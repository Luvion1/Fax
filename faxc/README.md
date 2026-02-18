# Fax Compiler

Fax is a modern systems programming language with static typing and garbage collection.

![MSRV](https://img.shields.io/badge/rustc-1.75+-blue.svg)
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

## Features

- **Static Typing**: Type inference with comprehensive checking
- **Garbage Collection**: Low-latency concurrent GC (FGC)
- **Zero-Cost Abstractions**: Compile-time optimizations
- **Memory Safety**: No null pointers, no use-after-free
- **Modern Tooling**: Package manager, debugger, profiler

## Quick Start

```bash
# Build the compiler
./scripts/build.sh

# Compile a Fax program
./target/release/faxc input.fax -o output

# Run tests
./scripts/test.sh
```

## Example

```fax
fn main() {
    let message = "Hello, Fax!";
    print(message);
}
```

## Documentation

- [Language Specification](docs/spec/grammar.md)
- [Architecture Overview](docs/arch/overview.md)
- [Compilation Pipeline](docs/arch/pipeline.md)
- [Technology Stack](docs/arch/technology.md)

## Project Structure

```
faxc/
├── crates/          # Compiler crates
├── tools/           # Developer tools
├── std/             # Standard library
├── tests/           # Test suites
└── docs/            # Documentation
```

## License

MIT OR Apache-2.0