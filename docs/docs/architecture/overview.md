---
sidebar_position: 1
---

# Overview

Fax uses a **polyglot compiler pipeline** where each stage is implemented in the most suitable language.

## Pipeline

```
Source (.fax)
       │
       ▼
┌──────────────────┐
│     Lexer        │  Rust 1.93.0
│  Tokenization    │
└────────┬─────────┘
         │ JSON Tokens
         ▼
┌──────────────────┐
│     Parser       │  Zig 0.14.1
│   AST Building   │
└────────┬─────────┘
         │ JSON AST
         ▼
┌──────────────────┐
│   Sema           │  Haskell (GHC 9.6.6)
│ Type Checking    │
└────────┬─────────┘
         │ Validated AST
         ▼
┌──────────────────┐
│    Optimizer     │  Rust 1.93.0
│  AST-level Opts  │
└────────┬─────────┘
         │ Optimized AST
         ▼
┌──────────────────┐
│    Codegen       │  C++ (GCC 15.2.0)
│   LLVM IR Gen    │
└────────┬─────────┘
         │ LLVM IR
         ▼
┌──────────────────┐
│    Runtime       │  Zig 0.14.1
│   (FGC GC)       │
└──────────────────┘
```

## Components

| Component | Language | Purpose |
|-----------|----------|---------|
| Lexer | Rust 1.93.0 | Tokenization |
| Parser | Zig 0.14.1 | AST generation |
| Sema | Haskell (GHC 9.6.6) | Type checking |
| Optimizer | Rust 1.93.0 | Code optimization |
| Codegen | C++ (GCC 15.2.0) | LLVM IR |
| Runtime | Zig 0.14.1 | Execution & GC |

## Directory Structure

```
faxc/
├── packages/
│   ├── lexer/        # Rust - Tokenization
│   ├── parser/       # Zig - AST generation
│   ├── sema/         # Haskell - Type checking
│   ├── optimizer/    # Rust - Optimizations
│   ├── codegen/     # C++ - LLVM IR
│   ├── runtime/     # Zig - Execution
│   └── hub/         # Node.js - Orchestrator
```

## Hub (Orchestrator)

The Hub (`faxc/packages/hub/`) written in **Node.js 20.20.0** coordinates the entire pipeline.
