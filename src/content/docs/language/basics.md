---
title: Language Basics
description: Basic syntax and concepts in Fax
---

## Variables

Use `let` to declare variables:

```fax
let x = 10;           // Type inferred as i64
let name = "Fax";     // Type inferred as str
let flag: bool = true; // Explicit type annotation
```

Constants use `const`:

```fax
const PI = 3.14159;
const MAX_SIZE = 100;
```

## Primitive Types

| Type | Description | Example |
|------|-------------|---------|
| `i64` | 64-bit integer | `42`, `-17` |
| `i32` | 32-bit integer | `100` |
| `bool` | Boolean | `true`, `false` |
| `str` | String | `"hello"` |
| `void` | No return value | Function return type |
| `null` | Null value | `null` |

## Comments

```fax
// Single-line comment

/*
 * Multi-line comment
 * Can span multiple lines
 */
```

## Basic Operations

### Arithmetic

```fax
let sum = 10 + 20;
let diff = 50 - 10;
let product = 5 * 8;
let quotient = 100 / 4;
let remainder = 17 % 5;
```

### Comparison

```fax
let a = 10;
let b = 20;

let eq = a == b;      // false
let neq = a != b;     // true
let lt = a < b;       // true
let lte = a <= b;     // true
let gt = a > b;       // false
let gte = a >= b;     // false
```

### Logical

```fax
let x = true;
let y = false;

let and = x && y;     // false
let or = x || y;      // true
let not = !x;         // false
```

## Blocks

Code blocks use curly braces:

```fax
{
    let x = 10;
    let y = 20;
    print(x + y);
}
```

Blocks create new scopes:

```fax
let x = 10;
{
    let x = 20;       // Different 'x' in this scope
    print(x);         // Prints 20
}
print(x);             // Prints 10
```

## Print Function

The built-in `print` function outputs values:

```fax
print(42);            // Print number
print("Hello");       // Print string
print(true);          // Print boolean
```
