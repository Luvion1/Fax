---
title: Types
description: Type system in Fax
---

## Primitive Types

### Integer Types

Fax supports signed integers:

```fax
let a: i64 = 1000000;     // 64-bit integer (default)
let b: i32 = 100;         // 32-bit integer
```

### Boolean

```fax
let flag: bool = true;
let other: bool = false;

let result = flag && other;  // false
let inverted = !flag;        // false
```

### String

```fax
let message: str = "Hello, Fax!";
let empty: str = "";

// String concatenation
let greeting = "Hello" + ", " + "World!";
```

### Void

Used for functions that don't return a value:

```fax
fn sayHello(): void {
    print("Hello!");
}
```

### Null

```fax
let empty: null = null;
```

## Composite Types

### Arrays

```fax
// Array literal
let numbers = [1, 2, 3, 4, 5];

// Access elements
let first = numbers[0];  // 1
let second = numbers[1]; // 2
```

### Tuples

```fax
let point = (10, 20);
let x = point.0;  // 10
let y = point.1;  // 20
```

### Pointers

```fax
let x = 10;
let ptr = &x;      // Reference to x
let value = *ptr;  // Dereference: value = 10
```

## Type Inference

Fax can infer types from values:

```fax
let x = 10;           // Inferred as i64
let name = "Fax";     // Inferred as str
let flag = true;      // Inferred as bool
```

## Type Annotations

Explicit type annotations are required in certain contexts:

```fax
// Function parameters
fn add(a: i64, b: i64): i64 {
    return a + b;
}

// When inference is ambiguous
let value: i32 = 100;
```

## Type Safety

Fax is statically typed and prevents common errors:

```fax
let x = 10;           // i64
let y = "hello";      // str

// This will cause a type error:
// let z = x + y;     // Error: cannot add i64 and str
```
