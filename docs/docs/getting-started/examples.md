---
sidebar_position: 3
---

# Examples

## Hello World

```fax
fn main() {
    print("Hello, Fax!");
}
```

## Fibonacci

```fax
fn fib(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

fn main() {
    print(fib(10));
}
```

## Array Operations

```fax
fn main() {
    let arr = [1, 2, 3, 4, 5];
    
    // Access element
    print(arr[0]);
    
    // Update element
    arr[0] = 10;
    print(arr[0]);
    
    // Array length (using built-in)
    print(arr[0]);  // First element
}
```

## Struct with Methods

```fax
struct Counter {
    value: i64,
}

fn main() {
    let c = Counter { value: 0 };
    c.value = c.value + 1;
    print(c.value);
}
```

## Factorial

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

## More Examples

See the [examples directory](https://github.com/Luvion1/Fax/tree/main/examples) for more programs.
