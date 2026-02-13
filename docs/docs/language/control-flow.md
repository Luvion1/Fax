---
sidebar_position: 4
---

# Control Flow

## If / Elif / Else

```fax
fn main() {
    let x = 10;
    
    if x > 20 {
        print("big");
    } elif x > 10 {
        print("medium");
    } else {
        print("small");
    }
}
```

## While Loop

```fax
fn main() {
    let mut i = 0;
    
    while i < 5 {
        print(i);
        i = i + 1;
    }
}
```

## For Loop (Range)

```fax
fn main() {
    // 0 to 9 (exclusive)
    for i in 0..10 {
        print(i);
    }
    
    // With step
    for j in 0..100..10 {
        print(j);  // 0, 10, 20, ..., 90
    }
}
```

## Break and Continue

```fax
fn main() {
    let mut i = 0;
    
    while true {
        i = i + 1;
        
        if i == 5 {
            break;  // Exit loop
        }
        
        if i == 2 {
            continue;  // Skip iteration
        }
        
        print(i);
    }
}
```

## Match (Pattern Matching)

```fax
fn main() {
    let x = 2;
    
    match x {
        1 => print("one"),
        2 => print("two"),
        3 => print("three"),
        default => print("other"),
    }
}
```

### Exhaustiveness

All cases must be handled or a `default` case must be provided.
