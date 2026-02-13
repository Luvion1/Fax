---
sidebar_position: 1
---

# Basics

## Hello World

```fax
fn main() {
    print("Hello, Fax!");
}
```

## Variables

```fax
fn main() {
    // Immutable variable
    let x = 10;
    
    // Mutable variable
    let mut y = 0;
    y = y + 1;
}
```

## Data Types

| Type | Description | Example |
|------|-------------|---------|
| `i64` | 64-bit integer | `42` |
| `bool` | Boolean | `true`, `false` |
| `str` | String | `"hello"` |
| `void` | No value | - |

## Comments

```fax
// Single line comment

/*
 * Multi-line
 * comment
 */
```

## Print

```fax
fn main() {
    print("Hello");
    print(42);
    print(true);
}
```

## Operators

### Arithmetic

```fax
let a = 10 + 5;   // add
let b = 10 - 5;   // sub
let c = 10 * 5;   // mul
let d = 10 / 5;   // div
let e = 10 % 3;   // remainder
```

### Comparison

```fax
let eq = 10 == 10;   // equal
let ne = 10 != 5;    // not equal
let lt = 5 < 10;    // less than
let gt = 10 > 5;    // greater than
let le = 5 <= 10;   // less or equal
let ge = 10 >= 10; // greater or equal
```

### Logical

```fax
let and = true && false;  // logical and
let or = true || false;   // logical or
let not = !true;          // logical not
```
