---
title: Structs
description: Structs in Fax
---

## Defining Structs

Structs are declared with the `struct` keyword:

```fax
struct Point {
    x: i64,
    y: i64
}

struct Person {
    name: str,
    age: i64
}
```

## Creating Instances

```fax
fn main() {
    let p = Point {
        x: 10,
        y: 20
    };
    
    let person = Person {
        name: "Alice",
        age: 30
    };
}
```

## Field Access

```fax
fn main() {
    let p = Point { x: 10, y: 20 };
    
    print(p.x);  // 10
    print(p.y);  // 20
}
```

## Structs in Functions

```fax
fn distance(p1: Point, p2: Point): i64 {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    return dx * dx + dy * dy;
}

fn main() {
    let a = Point { x: 0, y: 0 };
    let b = Point { x: 3, y: 4 };
    
    print(distance(a, b));  // 25
}
```

## Nested Structs

```fax
struct Address {
    street: str,
    city: str
}

struct Person {
    name: str,
    address: Address
}

fn main() {
    let person = Person {
        name: "Alice",
        address: Address {
            street: "123 Main St",
            city: "NYC"
        }
    };
    
    print(person.address.city);  // NYC
}
```
