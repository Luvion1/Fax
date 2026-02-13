# Language Guide

Fax combines systems-level control with high-level ergonomics.

## Basics

### Variables
```fax
let x = 10;          // immutable
let mut y = 20;      // mutable
y = 30;
```

### Functions
```fax
fn add(a: i64, b: i64) -> i64 {
  return a + b;
}

fn main() {
  print(add(5, 10));
}
```

## Control Flow

### Conditionals
```fax
if x > 0 {
  print("positive");
} else {
  print("non-positive");
}
```

### Loops
```fax
// Range loop
for i in 0..5 {
  print(i);
}

// Conditional loop
let mut k = 0;
while k < 5 {
  print(k);
  k = k + 1;
}
```

## Data Structures

### Structs
```fax
struct User {
  id: i64
  balance: i64
}

fn main() {
  let u = User { id: 1, balance: 500 };
  print(u.balance);
}
```

### Arrays
```fax
let nums = [1, 2, 3];
print(nums[1]);
```

## Memory Management

Fax uses a generational GC. You can trigger it manually:

```fax
fn stress() {
  let mut i = 0;
  while i < 10000 {
    let _ = [1, 2, 3];
    if i % 1000 == 0 { collect_gc(); }
    i = i + 1;
  }
}
```
