---
sidebar_position: 2
---

# Types

## Primitive Types

```fax
let num: i64 = 42;        // 64-bit integer
let flag: bool = true;    // Boolean
let text: str = "hello";  // String
let nothing: void;        // No value
```

## Arrays

```fax
let numbers = [1, 2, 3, 4, 5];
let empty: [i64] = [];

// Access
let first = numbers[0];

// Update (mutable)
let mut arr = [1, 2, 3];
arr[0] = 10;
```

## Type Inference

```fax
// Type inferred from value
let x = 42;        // i64
let y = "hello";   // str
let z = true;      // bool
```

## Type Annotations

```fax
let x: i64 = 42;
let y: bool = true;
let arr: [i64] = [1, 2, 3];
```

## Type Conversion

Currently, Fax uses implicit conversions between numeric types in certain contexts.
