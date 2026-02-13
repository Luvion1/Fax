# Fax-lang User Guide

Welcome to Fax! This guide will teach you how to write programs in the Fax language.

## Basic Concepts

### Variables
Use `let` for immutable variables and `var` for mutable ones.
```fax
let pi = 3.14
var counter = 0
counter = counter + 1
```

### Functions
Functions are declared with `fn`. The `main` function is the entry point.
```fax
fn main() {
    print("Hello Fax")
}

fn add(a: int, b: int): int {
    return a + b
}
```

## Data Structures

### Structs
Structs are used to group related data. They are automatically managed by Fgc.
```fax
struct Point {
    x: int,
    y: int
}

fn main() {
    let p = Point { x: 10, y: 20 }
    print(p.x)
}
```

### Arrays
Arrays in Fax are dynamic and heap-allocated.
```fax
fn main() {
    let list = [1, 2, 3, 4]
    print(list[0])
}
```

## Advanced Features

### Recursion and Memory
Fax handles deep recursion efficiently. Because of Fgc, you can create complex tree structures or linked lists without manually freeing memory.

```fax
struct Node {
    val: int,
    next: Node
}

fn create_list(n: int): Node {
    if n == 0 { return null }
    return Node { val: n, next: create_list(n - 1) }
}
```

## Troubleshooting

### Performance
While Fgc is efficient, frequent large allocations can trigger GC cycles. Use `Std_io_collect_fgc()` if you want to force a collection at a specific time.

### Error Messages
Fax provides Rust-like error messages. If your code fails to compile, pay close attention to the `Sema Error` output, which indicates type mismatches.
