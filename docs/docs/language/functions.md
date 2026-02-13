---
sidebar_position: 3
---

# Functions

## Function Declaration

```fax
fn function_name(param_name: param_type) -> return_type {
    // body
    return value;
}
```

## Examples

### No Return Value

```fax
fn greet(name: str) -> void {
    print("Hello, " + name);
}

fn main() {
    greet("Fax");
}
```

### With Return Value

```fax
fn add(a: i64, b: i64) -> i64 {
    return a + b;
}

fn main() {
    let result = add(5, 3);
    print(result);  // 8
}
```

### Expression Body

```fax
fn square(x: i64) -> i64 {
    x * x
}

fn main() {
    print(square(5));  // 25
}
```

## Recursion

```fax
fn factorial(n: i64) -> i64 {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

fn main() {
    print(factorial(5));  // 120
}
```
