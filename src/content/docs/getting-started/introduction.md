---
title: Introduction
description: Get started with Fax programming language
---

Welcome to Fax - a high-performance polyglot programming language with a custom generational garbage collector.

## What is Fax?

Fax is a systems programming language that combines the best of modern language design with a unique polyglot compiler architecture. Each stage of the compilation pipeline is implemented in the most suitable language for that specific task.

## Key Features

- **Polyglot Compiler**: Lexer (Rust), Parser (Zig), Semantic Analyzer (Haskell), Optimizer (Rust), Code Generator (C++), Runtime (Zig)
- **Generational GC**: Custom FGC (Fax Garbage Collector) with nursery, multiple page sizes, and parallel marking
- **Modern Syntax**: Clean, readable syntax inspired by Rust, Go, and TypeScript
- **Type Safety**: Static typing with type inference
- **Pattern Matching**: Powerful `match` expressions
- **First-class Functions**: Functions as values
- **Module System**: Support for imports and code organization

## Next Steps

- [Installation](/Fax/getting-started/installation/) - Set up Fax on your system
- [Quick Start](/Fax/getting-started/quick-start/) - Write your first Fax program
- [Language Basics](/Fax/language/basics/) - Learn the fundamentals
