# Quick Tour of Fax

A brief introduction to the Fax programming language.

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [Basic Syntax](#basic-syntax)
3. [Variables and Types](#variables-and-types)
4. [Functions](#functions)
5. [Control Flow](#control-flow)
6. [Data Types](#data-types)
7. [Pattern Matching](#pattern-matching)
8. [Next Steps](#next-steps)

---

## Design Philosophy

Fax combines the best of functional and imperative programming:

- **Simplicity** - Clean, readable syntax inspired by Go
- **Safety** - Strong static typing inspired by Rust
- **Expressiveness** - Functional features inspired by OCaml/Haskell
- **Performance** - Native code via LLVM

---

## Basic Syntax

Fax uses a clean, minimal syntax:

```fax
// This is a comment

fn main() {
    // Statements don't require semicolons
    let x = 42
    println(x)
}
```

---

## Variables and Types

### Variable Declaration

```fax
// Immutable variable (default)
let x = 42

// Mutable variable
let mut y = 10
y = 20

// Type annotation (optional with inference)
let z: i32 = 42
```

### Built-in Types

```fax
// Integers
let a: i8 = 127
let b: i16 = 32767
let c: i32 = 2147483647
let d: i64 = 9223372036854775807

// Unsigned integers
let e: u8 = 255
let f: u32 = 4294967295

// Floating point
let g: f32 = 3.14
let h: f64 = 3.14159265359

// Other primitives
let i: bool = true
let j: char = 'A'
let k: str = "Hello"
```

---

## Functions

### Basic Functions

```fax
// Simple function
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Function with multiple expressions
fn calculate(x: i32) -> i32 {
    let y = x * 2
    let z = y + 1
    z  // Return value (no semicolon)
}
```

### Lambda Expressions

```fax
// Lambda with type inference
let double = |x| x * 2

// Lambda with type annotation
let add = |a: i32, b: i32| -> i32 { a + b }

// Higher-order function
fn map(list: List<i32>, f: |i32| -> i32) -> List<i32> {
    // Implementation
}
```

---

## Control Flow

### Conditionals

```fax
// if/else expression (returns a value)
let result = if x > 10 {
    "large"
} else {
    "small"
}

// if/else if/else
if x < 0 {
    println("negative")
} else if x == 0 {
    println("zero")
} else {
    println("positive")
}
```

### Loops

```fax
// while loop
while x < 10 {
    x = x + 1
}

// infinite loop
loop {
    if condition {
        break
    }
}

// for loop (over ranges)
for i in 0..10 {
    println(i)
}
```

---

## Data Types

### Structs

```fax
// Define a struct
struct Point {
    x: f64,
    y: f64,
}

// Create an instance
let p = Point { x: 3.0, y: 4.0 }

// Access fields
let x_coord = p.x
```

### Enums (Algebraic Data Types)

```fax
// Define an enum
enum Result {
    Ok(i32),
    Err(str),
}

// Create variants
let success = Result::Ok(42)
let error = Result::Err("Something went wrong")
```

### Tuples

```fax
// Create a tuple
let pair = (42, "answer")

// Destructure a tuple
let (number, label) = pair

// Access by index
let first = pair.0
```

---

## Pattern Matching

```fax
enum Option {
    Some(i32),
    None,
}

fn main() {
    let value = Option::Some(42)

    match value {
        Option::Some(n) => println("Got: " + n),
        Option::None => println("Nothing"),
    }

    // Pattern matching with guards
    match x {
        n if n < 0 => println("negative"),
        0 => println("zero"),
        n if n > 0 => println("positive"),
    }

    // Destructuring in patterns
    let point = Point { x: 3.0, y: 4.0 }

    match point {
        Point { x: 0, y: 0 } => println("origin"),
        Point { x, y } => println("point at (" + x + ", " + y + ")"),
    }
}
```

---

## Next Steps

You've completed the quick tour! Continue learning:

### Language Features
- [Type System Deep Dive](../language-guide/types.md)
- [Advanced Functions](../language-guide/functions.md)
- [Pattern Matching Guide](../language-guide/pattern-matching.md)
- [Modules and Visibility](../language-guide/modules.md)

### Practice
- [Examples](../../faxc/examples/) - Sample programs
- [Exercises](../language-guide/exercises.md) - Practice problems

### Reference
- [Language Specification](../../SPEC.md) - Complete grammar
- [Standard Library](../language-guide/std.md) - Built-in functions

---

<div align="center">

**Continue your Fax journey!** 🚀

</div>
