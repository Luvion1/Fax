# Contributing to Fax-lang

Thank you for your interest in contributing! Since Fax-lang is a polyglot compiler, please follow these guidelines.

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
