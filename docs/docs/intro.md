---
sidebar_position: 1
---

# Welcome to Fax

Fax is a high-performance **polyglot programming language** where each compilation stage is implemented in the most suitable language for that task.

## Why Fax?

- **Polyglot Design**: Use the best tool for each job (Rust, Zig, Haskell, C++)
- **Generational GC**: Custom garbage collector for predictable performance
- **Modern Syntax**: Clean, readable syntax inspired by Rust, Go, TypeScript
- **Type Safety**: Static typing with powerful type inference

## Quick Example

```fax
fn main() {
    let message = "Hello, Fax!";
    print(message);
}
```

## Key Features

| Feature | Description |
|---------|-------------|
| **Static Typing** | Type inference with full type safety |
| **Pattern Matching** | Exhaustive match expressions |
| **Control Flow** | if/elif/else, while, for loops |
| **Structs** | Data structures with field access |
| **GC** | Generational garbage collector |

## Architecture

The Fax compiler pipeline:

```
Source → Lexer → Parser → Sema → Optimizer → Codegen → Runtime
         (Rust)   (Zig)   (Haskell)  (Rust)     (C++)    (Zig)
```

## Get Started

Ready to dive in? Head to the [Getting Started](/docs/getting-started/installation) guide!

## Resources

- [GitHub Repository](https://github.com/Luvion1/Fax)
- [Releases](https://github.com/Luvion1/Fax/releases)
- [Language Guide](/docs/language/basics)
- [Architecture](/docs/architecture/overview)
