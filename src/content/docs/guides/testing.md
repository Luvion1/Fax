---
title: Testing Guide
description: Learn how to write and run tests in Fax
---

# Testing Guide

Fax includes a built-in testing framework to help you write reliable code.

## Writing Tests

Tests are functions that verify your code behaves correctly.

### Basic Test Structure

```fax
fn test_addition() {
    let result = add(2, 3);
    assert(result == 5, "2 + 3 should equal 5");
}

fn test_subtraction() {
    let result = subtract(10, 4);
    assert(result == 6, "10 - 4 should equal 6");
}
```

## The assert Function

The `assert` function checks if a condition is true.

```fax
assert(condition, "Error message if condition is false");
```

If the condition is false, the test fails and the error message is displayed.

## Test Organization

### Naming Conventions

- Test functions should start with `test_`
- Use descriptive names that explain what is being tested

```fax
fn test_calculator_adds_positive_numbers() {
    // Test code
}

fn test_calculator_handles_negative_numbers() {
    // Test code
}

fn test_calculator_throws_on_division_by_zero() {
    // Test code
}
```

### Grouping Related Tests

Organize tests logically:

```fax
// Math operation tests
fn test_add() {
    assert(add(1, 1) == 2);
    assert(add(-1, 1) == 0);
    assert(add(0, 0) == 0);
}

fn test_multiply() {
    assert(multiply(2, 3) == 6);
    assert(multiply(-2, 3) == -6);
    assert(multiply(0, 100) == 0);
}

// Edge case tests
fn test_large_numbers() {
    assert(add(1000000, 2000000) == 3000000);
}

fn test_zero() {
    assert(multiply(0, 5) == 0);
    assert(add(0, 0) == 0);
}
```

## Testing Different Scenarios

### Happy Path

Test normal, expected behavior:

```fax
fn test_sort_orders_correctly() {
    let arr = [3, 1, 4, 1, 5];
    let sorted = bubbleSort(arr);
    assert(sorted[0] == 1);
    assert(sorted[4] == 5);
}
```

### Edge Cases

Test boundary conditions:

```fax
fn test_sort_empty_array() {
    let arr = [];
    let sorted = bubbleSort(arr);
    assert(len(sorted) == 0);
}

fn test_sort_single_element() {
    let arr = [42];
    let sorted = bubbleSort(arr);
    assert(sorted[0] == 42);
}

fn test_sort_already_sorted() {
    let arr = [1, 2, 3, 4, 5];
    let sorted = bubbleSort(arr);
    assert(sorted[0] == 1);
    assert(sorted[4] == 5);
}
```

### Error Cases

Test error handling:

```fax
fn test_division_by_zero() {
    let result = divide(10, 0);
    assert(result == 0, "Should return 0 on division by zero");
}

fn test_search_not_found() {
    let arr = [1, 2, 3];
    let index = binarySearch(arr, 99);
    assert(index == -1, "Should return -1 when element not found");
}
```

## Running Tests

### Run All Tests

```bash
python3 faxt/main.py test
```

### Run Specific Test File

```bash
python3 faxt/main.py test tests/calculator_test.fax
```

### Run Tests Matching Pattern

```bash
python3 faxt/main.py test tests/*_test.fax
```

## Test Output

When you run tests, you'll see output like:

```
Running tests...

✓ test_addition
✓ test_subtraction
✓ test_multiplication
✓ test_division
✓ test_large_numbers

5 tests passed, 0 tests failed
```

If a test fails:

```
✓ test_addition
✗ test_division
  Error: 10 / 0 should equal 0
  Expected: 0
  Actual: error

4 tests passed, 1 test failed
```

## Example: Testing a Calculator

```fax
// calculator.fax
fn add(a: i64, b: i64): i64 {
    return a + b;
}

fn divide(a: i64, b: i64): i64 {
    if (b == 0) {
        return 0;
    }
    return a / b;
}

// Tests
fn test_add_positive_numbers() {
    assert(add(2, 3) == 5);
    assert(add(10, 20) == 30);
}

fn test_add_negative_numbers() {
    assert(add(-5, -3) == -8);
    assert(add(-10, 5) == -5);
}

fn test_add_zero() {
    assert(add(0, 0) == 0);
    assert(add(5, 0) == 5);
    assert(add(0, 5) == 5);
}

fn test_divide_normal() {
    assert(divide(10, 2) == 5);
    assert(divide(100, 4) == 25);
}

fn test_divide_by_zero() {
    assert(divide(10, 0) == 0);
    assert(divide(0, 0) == 0);
}

fn test_divide_with_remainder() {
    assert(divide(10, 3) == 3);  // Integer division
    assert(divide(17, 5) == 3);
}

fn main() {
    // Run all tests automatically when in test mode
    test_add_positive_numbers();
    test_add_negative_numbers();
    test_add_zero();
    test_divide_normal();
    test_divide_by_zero();
    test_divide_with_remainder();
    
    print("All tests passed!");
}
```

## Best Practices

1. **Test One Thing**: Each test should verify a single concept
2. **Descriptive Names**: Test names should explain what's being tested
3. **Independent Tests**: Tests shouldn't depend on each other
4. **Test Edge Cases**: Don't just test the happy path
5. **Fast Tests**: Keep tests quick to run
6. **Readable Assertions**: Use clear error messages

## Test-Driven Development (TDD)

Fax supports TDD workflow:

1. **Write a failing test**
2. **Write minimal code to pass**
3. **Refactor**
4. **Repeat**

Example:

```fax
// Step 1: Write failing test
fn test_factorial() {
    assert(factorial(5) == 120);
}

// Step 2: Write minimal implementation
fn factorial(n: i64): i64 {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

// Step 3: Add more tests
fn test_factorial_edge_cases() {
    assert(factorial(0) == 1);
    assert(factorial(1) == 1);
    assert(factorial(10) == 3628800);
}
```

## Next Steps

- Learn about [Benchmarking](/Fax/guides/benchmarking/)
- Read about [Error Handling](/Fax/guides/error-handling/)
- Check out [Performance Tips](/Fax/guides/performance/)
