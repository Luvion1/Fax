---
title: Pattern Matching
description: Pattern matching with match expressions
---

## Match Expressions

Fax supports pattern matching with the `match` keyword:

```fax
fn main() {
    let x = 2;
    
    match x {
        case 1: { print("One"); }
        case 2: { print("Two"); }
        case 3: { print("Three"); }
        default: { print("Other"); }
    }
}
```

## Match with Multiple Cases

```fax
fn main() {
    let day = "Monday";
    
    match day {
        case "Saturday": { print("Weekend!"); }
        case "Sunday": { print("Weekend!"); }
        default: { print("Weekday"); }
    }
}
```

## Match with Expressions

```fax
fn getGrade(score: i64): str {
    match score {
        case 90..100: { return "A"; }
        case 80..89: { return "B"; }
        case 70..79: { return "C"; }
        case 60..69: { return "D"; }
        default: { return "F"; }
    }
}

fn main() {
    print(getGrade(85));  // B
}
```

## Exhaustive Matching

The compiler checks that all cases are covered:

```fax
fn main() {
    let x = 1;
    
    match x {
        case 1: { print("One"); }
        case 2: { print("Two"); }
        // Compiler will warn if default is missing
        default: { print("Other"); }
    }
}
```

## Match with Boolean Conditions

```fax
fn main() {
    let x = 10;
    let y = 20;
    
    match x {
        case _ if (x > y): { print("x is greater"); }
        case _ if (x < y): { print("x is smaller"); }
        default: { print("x equals y"); }
    }
}
```
