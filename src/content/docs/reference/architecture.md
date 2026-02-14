---
title: Architecture
description: Fax compiler architecture overview
---

## Overview

Fax uses a unique polyglot compiler design where each compilation stage is implemented in the most suitable language for that specific task.

## Compiler Pipeline

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

## Components

### Lexer (Rust)
- **Location**: `faxc/packages/lexer/`
- **Purpose**: Tokenize source code into tokens
- **Features**: String literals, escape sequences, comments

### Parser (Zig)
- **Location**: `faxc/packages/parser/`
- **Purpose**: Build AST from tokens
- **Features**: Declarations, statements, expressions

### Semantic Analyzer (Haskell)
- **Location**: `faxc/packages/sema/`
- **Purpose**: Type checking and validation
- **Features**: Type inference, control flow analysis

### Optimizer (Rust)
- **Location**: `faxc/packages/optimizer/`
- **Purpose**: Code optimization
- **Features**: Constant folding, dead code elimination

### Code Generator (C++)
- **Location**: `faxc/packages/codegen/`
- **Purpose**: Generate LLVM IR
- **Features**: Pointer handling, GC integration

### Runtime (Zig)
- **Location**: `faxc/packages/runtime/`
- **Purpose**: Execute compiled code
- **Features**: FGC (Fax Garbage Collector)
