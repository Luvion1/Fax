# Contributing to Fax-lang

Thank you for your interest in contributing! Since Fax-lang is a polyglot compiler, please follow these guidelines.

## Project Structure

The Fax project is organized as follows:
- `faxc/` - Main compiler components
  - `packages/` - Individual compiler stages
    - `lexer/` - Rust-based lexical analyzer
    - `parser/` - Zig-based parser
    - `sema/` - Haskell-based semantic analyzer
    - `optimizer/` - Rust-based optimizer
    - `codegen/` - C++-based code generator
    - `runtime/` - Zig-based garbage collector runtime
    - `hub/` - TypeScript-based orchestration layer
- `tests/` - Test files and examples
- `docs/` - Documentation files
- `examples/` - Example Fax programs
- `std/` - Standard library

## Development Setup

You will need the following tools installed:
- **Node.js** (v20+)
- **Rust** (Stable, via rustup)
- **Zig** (v0.13.0+)
- **GHC** (Haskell Compiler, v9.6+)
- **Python** (v3.10+)
- **Clang/LLVM** (v14+)

## Building & Testing

The entire pipeline is orchestrated via the TypeScript Hub.

1. Install dependencies:
   ```bash
   cd faxc
   npm install
   ```

2. Run a smoke test:
   ```bash
   npm start tests/fax/example.fax
   ```

## Component Guidelines

- **Lexer (Rust)**: Run `cargo clippy` before committing.
- **Parser (Zig)**: Run `zig fmt`.
- **Semantic (Haskell)**: Ensure `ghc` can compile `sema.hs`.
- **Hub (TS)**: Keep logs clean. Use `this.printStep`.

## Pull Requests

Please use the provided PR template and ensure the CI passes.
