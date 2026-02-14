---
title: Fax vs Other Languages
description: How Fax compares to other programming languages
---

# Fax vs Other Languages

Fax draws inspiration from many languages while offering unique features. Here's how it compares.

## Fax vs Rust

### Similarities
- Zero-cost abstractions
- Memory safety
- Pattern matching
- Strong static typing

### Differences

| Feature | Fax | Rust |
|---------|-----|------|
| Memory Management | Generational GC | Ownership + Borrowing |
| Learning Curve | Gentler | Steeper |
| Compile Times | Faster | Slower |
| Pattern Matching | match/case | match |
| Syntax | C-like | Unique |

**When to choose Fax:** Rapid prototyping, faster development cycles
**When to choose Rust:** Maximum control, no runtime

## Fax vs Go

### Similarities
- Fast compile times
- Clean syntax
- Built-in concurrency (planned for Fax)
- Garbage collection

### Differences

| Feature | Fax | Go |
|---------|-----|-----|
| Type System | Static with inference | Static, explicit |
| Generics | Full support | Limited (in Go 1.18+) |
| Error Handling | Pattern matching | Explicit returns |
| Inheritance | Structs + Traits (planned) | Interfaces |

**When to choose Fax:** Type safety, modern features
**When to choose Go:** Ecosystem, team familiarity

## Fax vs Python

### Similarities
- Clean, readable syntax
- Beginner-friendly
- Multiple return values

### Differences

| Feature | Fax | Python |
|---------|-----|--------|
| Performance | Compiled, fast | Interpreted, slower |
| Type Safety | Static | Dynamic |
| Runtime | Minimal | Heavy |
| GC | Generational | Reference counting + GC |

**When to choose Fax:** Performance-critical applications
**When to choose Python:** Data science, rapid scripting

## Fax vs TypeScript

### Similarities
- Modern syntax
- Type inference
- Pattern matching
- Great tooling

### Differences

| Feature | Fax | TypeScript |
|---------|-----|------------|
| Compilation | Native | Transpiles to JS |
| Runtime | Standalone | Requires Node.js/Browser |
| Type System | Sound | Gradual |
| Performance | Native speed | JavaScript speed |

**When to choose Fax:** System programming, CLIs
**When to choose TypeScript:** Web applications

## Fax vs C/C++

### Similarities
- Compiled to native code
- Predictable performance
- Low-level control (with planned features)

### Differences

| Feature | Fax | C/C++ |
|---------|-----|-------|
| Memory Safety | Guaranteed | Manual |
| Memory Management | GC | Manual/Smart pointers |
| Undefined Behavior | Prevented | Common |
| Build System | Built-in | Complex (CMake, etc.) |

**When to choose Fax:** Safety-critical code, faster development
**When to choose C/C++:** Existing ecosystems, maximum control

## Fax vs Zig

### Similarities
- Comptime evaluation
- Manual memory control (optional)
- C interop
- Simple toolchain

### Differences

| Feature | Fax | Zig |
|---------|-----|-----|
| Memory Management | GC by default | Manual |
| Safety | Opt-out | Opt-in |
| Pattern Matching | Built-in | Switch expressions |
| Compilation | Polyglot pipeline | Self-hosted |

**When to choose Fax:** Faster development, safety by default
**When to choose Zig**: Maximum control, manual optimization

## Unique Fax Features

### Polyglot Compiler
Fax's compiler uses the best tool for each job:
- Lexer: Rust (performance)
- Parser: Zig (simplicity)
- Type Checker: Haskell (correctness)
- Optimizer: Rust (speed)
- Codegen: C++ (LLVM)
- Runtime: Zig (small footprint)

### Generational GC
Predictable pause times with nursery collection and parallel marking.

### Modern Syntax
Clean, expressive syntax combining the best of:
- Rust's pattern matching
- Go's simplicity
- TypeScript's type system

## When to Use Fax

### Good For
- ✅ Systems programming
- ✅ CLIs and developer tools
- ✅ High-performance applications
- ✅ Teaching programming
- ✅ Rapid prototyping

### Not Ideal For
- ❌ Web frontend (use TypeScript)
- ❌ Android/iOS apps (use Kotlin/Swift)
- ❌ Data science (use Python)
- ❌ Legacy system integration (use C/C++)

## Migration Guide

### From Python
```python
# Python
def add(a, b):
    return a + b

result = add(2, 3)
print(result)
```

```fax
// Fax
fn add(a: i64, b: i64): i64 {
    return a + b;
}

fn main() {
    let result = add(2, 3);
    print(result);
}
```

### From JavaScript/TypeScript
```typescript
// TypeScript
function greet(name: string): string {
    return `Hello, ${name}!`;
}
```

```fax
// Fax
fn greet(name: str): str {
    return "Hello, " + name + "!";
}
```

### From Rust
```rust
// Rust
fn factorial(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}
```

```fax
// Fax
fn factorial(n: i64): i64 {
    match n {
        case 0: { return 1; }
        case 1: { return 1; }
        default: { return n * factorial(n - 1); }
    }
}
```

## Summary

Fax occupies a unique position:
- **Safer than C/C++** with guaranteed memory safety
- **Faster than Python** with native compilation
- **Simpler than Rust** with GC-based memory management
- **More modern than Go** with advanced type system features

Choose Fax when you want systems programming performance with modern language conveniences.
