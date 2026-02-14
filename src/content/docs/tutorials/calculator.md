---
title: Building a Calculator
description: Learn Fax by building a simple calculator application
---

# Building a Calculator in Fax

In this tutorial, we'll build a simple calculator that demonstrates core Fax concepts including functions, control flow, and error handling.

## What You'll Learn

- Defining functions with parameters and return types
- Using match expressions for control flow
- Handling edge cases (like division by zero)
- Basic arithmetic operations

## Getting Started

Create a new file called `calculator.fax`:

```fax
// Basic arithmetic functions
fn add(a: i64, b: i64): i64 {
    return a + b;
}

fn subtract(a: i64, b: i64): i64 {
    return a - b;
}

fn multiply(a: i64, b: i64): i64 {
    return a * b;
}
```

## Handling Division Safely

Division requires special handling to avoid division by zero:

```fax
fn divide(a: i64, b: i64): i64 {
    if (b == 0) {
        print("Error: Cannot divide by zero!");
        return 0;
    }
    return a / b;
}
```

## Advanced Operations

Let's add power and factorial functions:

```fax
fn power(base: i64, exp: i64): i64 {
    if (exp == 0) {
        return 1;
    }
    
    let result = 1;
    let i = 0;
    while (i < exp) {
        result = result * base;
        i = i + 1;
    }
    return result;
}

fn factorial(n: i64): i64 {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}
```

## Creating a Calculator Interface

Now let's create a function that handles different operations:

```fax
fn calculate(operation: str, a: i64, b: i64): i64 {
    match operation {
        case "+": { return add(a, b); }
        case "-": { return subtract(a, b); }
        case "*": { return multiply(a, b); }
        case "/": { return divide(a, b); }
        case "^": { return power(a, b); }
        default: {
            print("Unknown operation: " + operation);
            return 0;
        }
    }
}
```

## Putting It All Together

```fax
fn main() {
    print("=== Fax Calculator ===");
    print("");
    
    // Test basic operations
    print("10 + 5 = " + calculate("+", 10, 5));
    print("20 - 8 = " + calculate("-", 20, 8));
    print("7 * 6 = " + calculate("*", 7, 6));
    print("100 / 4 = " + calculate("/", 100, 4));
    
    // Test power
    print("2 ^ 8 = " + calculate("^", 2, 8));
    
    // Test factorial
    print("5! = " + factorial(5));
    
    // Test error handling
    print("10 / 0 = ");
    calculate("/", 10, 0);
}
```

## Running the Calculator

Save the file and run it:

```bash
python3 faxt/main.py run calculator.fax
```

## Expected Output

```
=== Fax Calculator ===

10 + 5 = 15
20 - 8 = 12
7 * 6 = 42
100 / 4 = 25
2 ^ 8 = 256
5! = 120
10 / 0 = 
Error: Cannot divide by zero!
```

## Challenge Exercises

1. **Add Modulo**: Implement a modulo operator (`%`)
2. **Add Absolute Value**: Create an `abs()` function
3. **Add Minimum/Maximum**: Implement `min()` and `max()` functions
4. **Chain Operations**: Allow operations like `calculate("+", calculate("*", 2, 3), 4)`

## Key Takeaways

- Functions in Fax have explicit return types
- Use `match` for clean multi-way branching
- Always handle edge cases (like division by zero)
- Recursion works just like in other languages

## Next Steps

Now that you've built a calculator, try:
- [Building a Todo List](/Fax/tutorials/todo-list/)
- [Working with Data Structures](/Fax/tutorials/data-structures/)
