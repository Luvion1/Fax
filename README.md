# Fax Programming Language

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.0.1-blue.svg)](https://github.com/Luvion1/Fax/releases)
[![GitHub Pages](https://img.shields.io/badge/GitHub-Pages-active-green.svg)](https://luvion1.github.io/Fax/)
[![Tests](https://img.shields.io/badge/Tests-10%2C000%2B-success-green.svg)]()

**Fax** is a high-performance, polyglot programming language compiler. Each stage of the compilation pipeline is implemented in the most suitable language for that specific task.

## Table of Contents

- [About](#about)
- [Features](#features)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Documentation](#documentation)
- [Building](#building)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

## About

Fax is a systems programming language that combines:
- **Polyglot Compiler Design**: Each compilation stage uses the best tool for the job
- **Generational Garbage Collector**: Custom FGC (Fax Garbage Collector) for predictable memory management
- **Modern Syntax**: Clean, readable syntax inspired by Rust, Go, and TypeScript
- **Module System**: Support for imports and code organization
- **Extensive Testing**: 10,000+ test cases for reliability

### Why Fax?

- **Performance**: Optimized compiler pipeline with multiple optimization stages
- **Developer Experience**: Clear error messages with smart suggestions
- **Transparency**: Direct access to GC control for performance tuning
- **Educational**: Learn compiler construction through well-documented source

## Features

### Language Features
- **Type System**: Static typing with type inference
  - Primitive types: `i64`, `bool`, `str`, `void`
  - Composite types: arrays, structs, pointers, functions
- **Control Flow**: `if/elif/else`, `while`, `for` loops
- **Pattern Matching**: `match/case/default` expressions
- **Functions**: First-class functions with parameter support
- **Structs**: Data structures with field access
- **Memory Safety**: Bounds checking, null safety

### Compiler Pipeline
1. **Lexer** (Rust) - Tokenization
2. **Parser** (Zig) - AST generation
3. **Semantic Analyzer** (Haskell) - Type checking
4. **Optimizer** (Rust) - Code optimization
5. **Code Generator** (C++) - LLVM IR generation
6. **Runtime** (Zig) - Execution with GC

### Error Detection
- Type mismatch errors
- Undefined symbols
- Missing returns
- Unreachable code detection
- Pattern exhaustiveness checking
- And 20+ more error types

## Quick Start

### Prerequisites

- **Rust** (for lexer, optimizer)
- **Zig** (for parser, runtime)
- **GHC** (for semantic analyzer)
- **C++ Compiler** (for code generator)
- **Node.js** (for tooling)

### Installation

```bash
# Clone the repository
git clone https://github.com/Luvion1/Fax.git
cd Fax

# Install dependencies
npm install

# Build all components
make build

# Or use the CLI directly
python3 faxt/main.py run examples/hello.fax
```

### Create Your First Program

```fax
fn main() {
    let message = "Hello, Fax!";
    print(message);
}
```

Run it:
```bash
python3 faxt/main.py run your_file.fax
```

Or use the CLI:
```bash
ln -sf $(pwd)/faxt/main.py /usr/local/bin/faxt
faxt run examples/hello.fax
```

## Architecture

```
Source Code (.fax)
       │
       ▼
┌──────────────────┐
│    Lexer         │  Rust
│  (Tokenization)  │
└────────┬─────────┘
         │ JSON Tokens
         ▼
┌──────────────────┐
│    Parser        │  Zig
│  (AST Building)  │
└────────┬─────────┘
         │ JSON AST
         ▼
┌──────────────────┐
│   Sema           │  Haskell
│ (Type Checking)  │
└────────┬─────────┘
         │ Validated AST
         ▼
┌──────────────────┐
│   Optimizer      │  Rust
│ (Optimizations)  │
└────────┬─────────┘
         │ Optimized AST
         ▼
┌──────────────────┐
│   Codegen        │  C++
│  (LLVM IR)       │
└────────┬─────────┘
         │ LLVM IR
         ▼
┌──────────────────┐
│   Runtime        │  Zig
│   (Execution)    │
└──────────────────┘
```

For detailed architecture documentation, see [ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Documentation

| Document | Description |
|----------|-------------|
| [Getting Started](docs/getting-started.md) | Quick start guide |
| [Language Guide](docs/language.md) | Syntax and language features |
| [Toolchain Manual](docs/tooling.md) | `faxt` CLI reference |
| [Architecture](docs/ARCHITECTURE.md) | Compiler internals |
| [Memory Management](docs/memory.md) | FGC garbage collector |
| [Semantic Analyzer](docs/semantic-analyzer.md) | Sema module reference |
| [API Reference](docs/api.md) | Standard library |

## Building

### Build All Components

```bash
make build
```

### Build Individual Components

```bash
# Lexer (Rust)
cd faxc/packages/lexer && cargo build --release

# Parser (Zig)
cd faxc/packages/parser && zig build

# Semantic Analyzer (Haskell)
cd faxc/packages/sema && ghc -o sema src/Main.hs

# Optimizer (Rust)
cd faxc/packages/optimizer && cargo build --release

# Code Generator (C++)
cd faxc/packages/codegen && mkdir -p build && cd build && cmake .. && make

# Runtime (Zig)
cd faxc/packages/runtime && zig build
```

### Using the Toolchain

```bash
# Check types
python3 faxt/main.py check <file.fax>

# Build
python3 faxt/main.py build <file.fax>

# Run
python3 faxt/main.py run <file.fax>

# Interactive REPL
python3 faxt/main.py repl
```

## Testing

### Run All Tests

```bash
make test
```

### Run Specific Test Categories

```bash
# Arithmetic tests
python3 faxt/main.py test tests/arithmetic_*.fax

# Logic tests
python3 faxt/main.py test tests/logic_*.fax

# Custom test
python3 faxt/main.py run your_test.fax
```

### Test Files Location

- `tests/` - Main test suite (10,000+ tests)
- `faxc/tests/fax/` - Integration tests
- `examples/` - Example programs

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

1. Install all prerequisites
2. Fork the repository
3. Create a feature branch
4. Make your changes
5. Run tests
6. Submit a PR

### Code Style

- **Rust**: Follow `rustfmt` guidelines
- **Zig**: Use Zig's formatting standards
- **Haskell**: Follow standard Haskell conventions
- **C++**: Use `clang-format`

## Community

- [GitHub Issues](https://github.com/Luvion1/Fax/issues) - Report bugs
- [Discussions](https://github.com/Luvion1/Fax/discussions) - Ask questions
- [Releases](https://github.com/Luvion1/Fax/releases) - Version history

## License

Fax is released under the **MIT License**. See [LICENSE](LICENSE) for details.

---

**Website**: https://luvion1.github.io/Fax/  
**GitHub**: https://github.com/Luvion1/Fax  
**Version**: 0.0.1
