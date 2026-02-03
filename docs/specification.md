# Fax Programming Language Specification

## Table of Contents
1. [About Fax](#about-fax)
2. [Key Features](#key-features)
3. [Basic Syntax](#basic-syntax)
4. [Data Types](#data-types)
5. [Control Structures](#control-structures)
6. [Functions](#functions)
7. [Modules and Namespaces](#modules-and-namespaces)
8. [Traits and Generics](#traits-and-generics)
9. [Pattern Matching](#pattern-matching)
10. [Error Handling](#error-handling)
11. [Memory Management](#memory-management)
12. [Standard Library](#standard-library)
13. [Toolchain](#toolchain)
14. [Multi-Language Integration](#multi-language-integration)

## About Fax

Fax is a modern programming language designed to provide a perfect combination of ease of use, high safety, and high performance. It combines the best elements of various programming paradigms and adopts a clean, expressive modern syntax.

### Design Philosophy
- **Maintainability**: Clean and easy-to-understand syntax.
- **Safety**: Type safety and memory safety without significant runtime overhead.
- **Performance**: Compiled to native code with high performance.
- **Flexibility**: Supports declarative, procedural, and functional paradigms.

## Key Features

### 1. Modern and Expressive Syntax
```fax
// Single-line comment
/* Multi-line
   comment */

// Basic function
fn main() {
    let greeting = "Hello, World!"
    io.println(greeting)
}

// Function with type annotation
fn add(a: Int, b: Int): Int {
    a + b
}
```

### 2. Static Typing with Type Inference
```fax
// Type inference
let x = 42        // x: Int
let name = "Bob"  // name: String

// Explicit type annotation
let pi: Float = 3.14159
let is_valid: Bool = true
```

### 3. Compiled with Fgc (Garbage Collector)
```fax
// No need to worry about manual memory management
fn example() {
    let data = vec![1, 2, 3, 4, 5]  // allocated on heap
    let shared = data               // shared reference
    // Fgc will clean up when no longer in use
}
```

## Basic Syntax

### Variables and Constants
```fax
// Immutable by default
let x = 42
let name = "Alice"

// Mutable variable
var counter = 0
counter += 1

// Constant
const MAX_SIZE: Int = 100
```

### Operators
```fax
// Arithmetic
let sum = a + b
let diff = a - b
let prod = a * b
let quot = a / b
let rem = a % b

// Comparison
let equal = a == b
let greater = a > b
let less_eq = a <= b

// Logical
let and_result = a && b
let or_result = a || b
let not_result = !a
```

## Data Types

### Primitive Types
```fax
// Integers
let byte_val: Byte = 255
let int_val: Int = 42
let long_val: Long = 1000000

// Floating point
let float_val: Float = 3.14
let double_val: Double = 3.14159265359

// Boolean
let is_true: Bool = true
```

### Collection Types
```fax
// Array (fixed size)
let arr: [Int; 3] = [1, 2, 3]

// Vector (dynamic size)
let vec: Vec<Int> = vec![1, 2, 3, 4, 5]

// HashMap
let map: HashMap<String, Int> = HashMap::new()
```

### Custom Types
```fax
// Struct
struct Person {
    name: String,
    age: Int,
}

// Enum
enum Color {
    Red,
    Green,
    Blue,
}
```

## Control Structures

### Conditional Statements
```fax
if x > 0 {
    io.println("Positive")
} else {
    io.println("Non-positive")
}

// Match expression
match color {
    Color::Red => io.println("Red"),
    _ => io.println("Other"),
}
```

### Loops
```fax
for i in 0..10 {
    io.println(i)
}

while condition {
    // do something
}
```

## Memory Management

### Fgc (Fax Garbage Collector)
Fax uses **Fgc** (Fax Garbage Collector), a modern tracing garbage collector implemented in Zig.

**Fgc Features:**
- **ZGC-inspired Coloring**: Uses pointer coloring techniques for efficient object state tracking.
- **Mark-Relocate Algorithm**: Moves live objects to eliminate memory fragmentation.
- **Load Barriers**: Ensures safe memory access while objects are being relocated.
- **Polyglot Ready**: Natively integrated with Rust, C++, and Zig components within the Fax pipeline.

```fax
// Memory managed automatically by Fgc
fn memory_example() {
    let data = vec![1, 2, 3, 4, 5]  // allocated on heap, managed by Fgc
    let shared_ref = data           // reference shared
    
    // Fgc will clean up memory when no more references point to this data.
}
```

## Multi-Language Integration

Fax supports seamless integration with multiple languages (Rust, Zig, C++, Haskell, Python) through its modular compiler architecture and C ABI compatibility.

## Toolchain

- **faxc**: The main compiler.
- **Fgc Runtime**: The Zig-based garbage collector and runtime library.