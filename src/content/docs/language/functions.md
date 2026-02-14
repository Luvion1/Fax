---
title: Functions
description: Functions in Fax
---

## Defining Functions

Functions are declared with the `fn` keyword:

```fax
fn greet() {
    print("Hello!");
}

fn main() {
    greet();
}
```

## Parameters

```fax
fn add(a: i64, b: i64): i64 {
    return a + b;
}

fn main() {
    let result = add(5, 3);
    print(result);  // 8
}
```

## Return Values

```fax
fn square(x: i64): i64 {
    return x * x;
}

fn getMessage(): str {
    return "Hello, Fax!";
}
```

## Void Functions

Functions that don't return a value:

```fax
fn sayHello(name: str): void {
    print("Hello, " + name);
}
```

## First-Class Functions

Functions can be stored in variables and passed as arguments:

```fax
fn apply(x: i64, f: fn(i64) -> i64): i64 {
    return f(x);
}

fn double(x: i64): i64 {
    return x * 2;
}

fn main() {
    let result = apply(5, double);
    print(result);  // 10
}
```

## Recursion

```fax
fn factorial(n: i64): i64 {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

fn main() {
    print(factorial(5));  // 120
}
```

## Main Function

Every Fax program needs a `main` function as entry point:

```fax
fn main() {
    // Program starts here
    print("Hello, World!");
}
```
