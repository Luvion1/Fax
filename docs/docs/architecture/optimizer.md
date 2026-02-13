---
sidebar_position: 5
---

# Optimizer

**Location**: `faxc/packages/optimizer/`

The Optimizer is implemented in **Rust 1.93.0** and performs AST-level optimizations.

## Features

- **Constant Folding**: Evaluate constant expressions
- **Dead Code Elimination**: Remove unreachable code
- **5 Optimization Levels**: 0-4

## Build

```bash
cd faxc/packages/optimizer
cargo build --release
```
