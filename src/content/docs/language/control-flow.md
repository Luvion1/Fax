---
title: Control Flow
description: Control flow statements in Fax
---

## If Statements

```fax
fn main() {
    let x = 10;
    
    if (x > 5) {
        print("Greater than 5");
    }
}
```

## If-Else

```fax
fn main() {
    let x = 10;
    
    if (x > 5) {
        print("Greater than 5");
    } else {
        print("Not greater than 5");
    }
}
```

## If-Elif-Else

```fax
fn main() {
    let score = 85;
    
    if (score >= 90) {
        print("A");
    } elif (score >= 80) {
        print("B");
    } elif (score >= 70) {
        print("C");
    } else {
        print("F");
    }
}
```

## While Loops

```fax
fn main() {
    let i = 0;
    
    while (i < 5) {
        print(i);
        i = i + 1;
    }
}
```

## For Loops

```fax
fn main() {
    for (let i = 0; i < 5; i = i + 1) {
        print(i);
    }
}
```

## Break and Continue

```fax
fn main() {
    let i = 0;
    
    while (i < 10) {
        i = i + 1;
        
        if (i == 3) {
            continue;  // Skip 3
        }
        
        if (i == 7) {
            break;     // Stop at 7
        }
        
        print(i);  // Prints 1, 2, 4, 5, 6
    }
}
```
