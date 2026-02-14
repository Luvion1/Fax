---
title: Performance Optimization
description: Tips and techniques for writing fast Fax code
---

# Performance Optimization

This guide covers techniques for writing high-performance Fax code.

## Understanding Fax's Performance Characteristics

Fax is designed for performance:
- **Zero-cost abstractions**: High-level features compile to efficient machine code
- **Generational GC**: Predictable pause times under 10ms
- **LLVM backend**: State-of-the-art optimizations
- **Static typing**: No runtime type checks

## General Guidelines

### 1. Prefer Stack Allocation

Stack allocation is faster than heap allocation:

```fax
// Fast - stack allocated
let x = 42;
let arr = [1, 2, 3, 4, 5];

// Slower - heap allocated (if using dynamic allocation)
let largeArray = allocateArray(1000000);
```

### 2. Use const for Compile-Time Values

```fax
// Better - computed at compile time
const PI = 3.14159;
const MAX_SIZE = 1000;

// OK but computed at runtime
let pi = 3.14159;
```

### 3. Minimize Heap Allocations

Each heap allocation triggers GC work:

```fax
// Inefficient - many allocations
fn inefficient() {
    let i = 0;
    while (i < 1000) {
        let s = "Iteration " + i;  // New string each time
        print(s);
        i = i + 1;
    }
}

// Better - reuse memory
fn better() {
    let i = 0;
    let prefix = "Iteration ";
    while (i < 1000) {
        print(prefix + i);  // Reuse prefix
        i = i + 1;
    }
}
```

## Algorithm Optimization

### Choose the Right Data Structure

| Operation | Array | Linked List | Notes |
|-----------|-------|-------------|-------|
| Access by index | O(1) | O(n) | Arrays win |
| Insert at end | O(1)* | O(1) | *Amortized |
| Insert at beginning | O(n) | O(1) | Linked list wins |
| Search | O(n) | O(n) | Same |

```fax
// Good - array for random access
let arr = [1, 2, 3, 4, 5];
let x = arr[1000];  // O(1)

// Bad - array for frequent insertions at front
fn bad() {
    let arr = [];
    let i = 0;
    while (i < 1000) {
        // This is O(n) each time!
        insertAtBeginning(arr, i);
        i = i + 1;
    }
}
```

### Cache-Friendly Access Patterns

Access memory sequentially for better cache performance:

```fax
// Good - sequential access
fn good(arr: []i64): i64 {
    let sum = 0;
    let i = 0;
    while (i < len(arr)) {
        sum = sum + arr[i];  // Sequential access
        i = i + 1;
    }
    return sum;
}

// Bad - random access
fn bad(matrix: [][]i64): i64 {
    let sum = 0;
    let col = 0;
    while (col < len(matrix[0])) {
        let row = 0;
        while (row < len(matrix)) {
            sum = sum + matrix[row][col];  // Column-major is slow
            row = row + 1;
        }
        col = col + 1;
    }
    return sum;
}
```

## Memory Management

### Understanding GC Behavior

```fax
// Trigger GC manually when appropriate
if (memoryUsage() > threshold) {
    gc();
}

// Or let it run automatically
```

### Avoid Memory Leaks

```fax
// Bad - circular reference (if references existed)
// (Note: Fax handles this, but good to be aware)

// Good - clear references when done
fn process() {
    let data = loadLargeData();
    processData(data);
    // data goes out of scope, can be collected
}
```

## Function Optimization

### Inline Small Functions

The compiler automatically inlines small functions, but you can help:

```fax
// Good candidates for inlining
fn square(x: i64): i64 {
    return x * x;
}

fn isEven(n: i64): bool {
    return n % 2 == 0;
}
```

### Avoid Recursion for Deep Operations

```fax
// Risky - stack overflow for large n
fn recursiveSum(n: i64): i64 {
    if (n <= 0) {
        return 0;
    }
    return n + recursiveSum(n - 1);
}

// Better - iterative
fn iterativeSum(n: i64): i64 {
    let sum = 0;
    let i = 1;
    while (i <= n) {
        sum = sum + i;
        i = i + 1;
    }
    return sum;
}
```

## Loop Optimization

### Loop Unrolling

```fax
// Standard loop
fn sumArray(arr: []i64): i64 {
    let sum = 0;
    let i = 0;
    while (i < len(arr)) {
        sum = sum + arr[i];
        i = i + 1;
    }
    return sum;
}

// Unrolled loop (4x)
fn sumArrayUnrolled(arr: []i64): i64 {
    let sum = 0;
    let i = 0;
    let n = len(arr);
    
    // Process 4 elements at a time
    while (i + 3 < n) {
        sum = sum + arr[i] + arr[i + 1] + arr[i + 2] + arr[i + 3];
        i = i + 4;
    }
    
    // Handle remainder
    while (i < n) {
        sum = sum + arr[i];
        i = i + 1;
    }
    
    return sum;
}
```

### Move Invariant Code Out of Loops

```fax
// Bad - computing length every iteration
fn bad(arr: []i64): i64 {
    let sum = 0;
    let i = 0;
    while (i < len(arr)) {  // len() called every time
        sum = sum + arr[i];
        i = i + 1;
    }
    return sum;
}

// Good - cache the length
fn good(arr: []i64): i64 {
    let sum = 0;
    let i = 0;
    let n = len(arr);  // Computed once
    while (i < n) {
        sum = sum + arr[i];
        i = i + 1;
    }
    return sum;
}
```

## Benchmarking

### Measure Performance

```bash
# Time a single run
time python3 faxt/main.py run program.fax

# Benchmark with multiple runs
faxt bench program.fax --runs 100
```

### Profile Your Code

```bash
# Generate performance profile
faxt run --profile program.fax

# View hotspot report
cat profile.txt
```

## Common Pitfalls

### 1. String Concatenation in Loops

```fax
// Slow - O(n²) total
fn slow(n: i64) {
    let result = "";
    let i = 0;
    while (i < n) {
        result = result + "x";  // New allocation each time
        i = i + 1;
    }
}

// Better - use array and join
fn better(n: i64) {
    let arr = [];
    let i = 0;
    while (i < n) {
        push(arr, "x");  // Amortized O(1)
        i = i + 1;
    }
    let result = join(arr, "");
}
```

### 2. Repeated Computations

```fax
// Bad - computing expensive operation multiple times
fn bad(x: i64): i64 {
    if (expensiveOp(x) > 10) {
        return expensiveOp(x) * 2;  // Computed twice!
    }
    return expensiveOp(x);  // And again!
}

// Good - cache the result
fn good(x: i64): i64 {
    let result = expensiveOp(x);  // Computed once
    if (result > 10) {
        return result * 2;
    }
    return result;
}
```

### 3. Unnecessary Copying

```fax
// Bad - copying large array
fn processCopy(arr: []i64): i64 {
    let copy = clone(arr);  // O(n) copy
    return sum(copy);
}

// Good - use reference
fn processRef(arr: []i64): i64 {
    return sum(arr);  // O(1) reference
}
```

## Best Practices Summary

1. **Profile First**: Don't optimize without measuring
2. **Algorithm Choice**: Right data structure beats micro-optimizations
3. **Memory Matters**: Minimize allocations and copies
4. **Cache is King**: Sequential access is faster than random
5. **Let LLVM Help**: Write clean code, trust the optimizer

## Further Reading

- [Testing Guide](/Fax/guides/testing/)
- [Architecture Overview](/Fax/reference/architecture/)
- [FGC Documentation](/Fax/reference/fgc/)
