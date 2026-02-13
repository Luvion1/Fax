---
sidebar_position: 5
---

# Structs

## Definition

```fax
struct Point {
    x: i64,
    y: i64,
}
```

## Creating Instances

```fax
fn main() {
    let p = Point { x: 10, y: 20 };
    print(p.x);  // 10
    print(p.y);  // 20
}
```

## Mutable Struct

```fax
fn main() {
    let mut p = Point { x: 10, y: 20 };
    p.x = 30;
    print(p.x);  // 30
}
```

## Nested Structs

```fax
struct Point {
    x: i64,
    y: i64,
}

struct Line {
    start: Point,
    end: Point,
}

fn main() {
    let line = Line {
        start: Point { x: 0, y: 0 },
        end: Point { x: 10, y: 10 },
    };
    print(line.start.x);
}
```
