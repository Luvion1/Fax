# Fax Language Guide

Comprehensive guide to the Fax programming language.

## Table of Contents

### Basics
- [Variables and Mutability](basics.md#variables-and-mutability)
- [Basic Types](basics.md#basic-types)
- [Operators](basics.md#operators)

### Functions
- [Function Definition](functions.md#function-definition)
- [Parameters and Return](functions.md#parameters-and-return)
- [Lambda Expressions](functions.md#lambda-expressions)
- [Higher-Order Functions](functions.md#higher-order-functions)

### Types
- [Type System Overview](types.md#overview)
- [Primitive Types](types.md#primitive-types)
- [Compound Types](types.md#compound-types)
- [Type Inference](types.md#type-inference)

### Control Flow
- [Conditionals](control-flow.md#conditionals)
- [Loops](control-flow.md#loops)
- [Pattern Matching](pattern-matching.md)

### Data Types
- [Structs](data-types.md#structs)
- [Enums](data-types.md#enums)
- [Tuples](data-types.md#tuples)
- [Arrays](data-types.md#arrays)

### Advanced Topics
- [Modules](modules.md)
- [Generics](advanced.md#generics)
- [Traits](advanced.md#traits)
- [Error Handling](advanced.md#error-handling)

---

## Quick Reference

### Hello World

```fax
fn main() {
    println("Hello, World!")
}
```

### Variables

```fax
let x = 42              // Immutable
let mut y = 10          // Mutable
let z: f64 = 3.14       // Type annotation
```

### Functions

```fax
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Pattern Matching

```fax
match value {
    Option::Some(n) => println(n),
    Option::None => println("nothing"),
}
```

### Structs

```fax
struct Point {
    x: f64,
    y: f64,
}

let p = Point { x: 3.0, y: 4.0 }
```

### Enums

```fax
enum Result {
    Ok(i32),
    Err(str),
}
```

---

## Language Features at a Glance

| Feature | Description | Status |
|---------|-------------|--------|
| Type Inference | Automatic type deduction | ✅ Implemented |
| Pattern Matching | Exhaustive match expressions | ✅ Implemented |
| Algebraic Data Types | Enums and structs | ✅ Implemented |
| First-class Functions | Functions as values | ✅ Implemented |
| Immutability by Default | Variables are immutable | ✅ Implemented |
| Garbage Collection | Automatic memory management | ✅ Implemented |
| Generics | Parametric polymorphism | 🚧 In Progress |
| Traits | Interface-like abstractions | 🚧 In Progress |
| Async/Await | Asynchronous programming | 📋 Planned |

---

## Code Examples

See the [`examples/`](../../faxc/examples/) directory for complete working examples:

| Example | Description |
|---------|-------------|
| `01_hello.fax` | Hello World |
| `02_variables.fax` | Variables and types |
| `03_functions.fax` | Functions and lambdas |
| `04_match.fax` | Pattern matching |
| `05_structs.fax` | Structs and methods |
| `06_enums.fax` | Enums and ADTs |

---

## Additional Resources

- [Language Specification](../../SPEC.md) - Complete grammar reference
- [Getting Started](../getting-started/) - Installation and basics
- [Compiler Documentation](../compiler/) - How the compiler works
- [RFCs](../rfcs/) - Design proposals and discussions

---

<div align="center">

**Master the Fax language!** 🚀

</div>
