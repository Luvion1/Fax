---
title: Quick Start
description: Write your first Fax program
---

## Your First Program

Create a file named `hello.fax`:

```fax
fn main() {
    print("Hello, World!");
}
```

Run it:

```bash
python3 faxt/main.py run hello.fax
```

Output:
```
Hello, World!
```

## Variables and Types

```fax
fn main() {
    // Type inference
    let x = 10;           // i64
    let name = "Fax";     // str
    
    // Explicit type annotation
    let flag: bool = true;
    
    print(x);
    print(name);
}
```

## Functions

```fax
fn add(a: i64, b: i64): i64 {
    return a + b;
}

fn main() {
    let result = add(5, 3);
    print(result);  // 8
}
```

## Control Flow

```fax
fn main() {
    let x = 10;
    
    if (x > 5) {
        print("Greater than 5");
    } elif (x == 5) {
        print("Equal to 5");
    } else {
        print("Less than 5");
    }
    
    // While loop
    let i = 0;
    while (i < 3) {
        print(i);
        i = i + 1;
    }
    
    // For loop
    for (let j = 0; j < 3; j = j + 1) {
        print(j);
    }
}
```

## Structs

```fax
struct Point {
    x: i64,
    y: i64
}

fn main() {
    let p = Point { x: 10, y: 20 };
    print(p.x);  // 10
    print(p.y);  // 20
}
```

## Pattern Matching

```fax
fn main() {
    let x = 2;
    
    match x {
        case 1: { print("One"); }
        case 2: { print("Two"); }
        default: { print("Other"); }
    }
}
```

## Next Steps

- Learn more about [language basics](/Fax/language/basics/)
- Explore [types and functions](/Fax/language/types/)
- Understand the [compiler architecture](/Fax/reference/architecture/)
