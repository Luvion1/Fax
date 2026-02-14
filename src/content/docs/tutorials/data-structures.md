---
title: Working with Data Structures
description: Learn how to use arrays, structs, and algorithms in Fax
---

# Working with Data Structures

This tutorial covers arrays, structs, and common algorithms in Fax. You'll learn how to organize and manipulate data effectively.

## Arrays in Fax

Arrays are ordered collections of elements with the same type.

### Creating Arrays

```fax
// Array literal
let numbers = [10, 20, 30, 40, 50];
let names = ["Alice", "Bob", "Charlie"];

// Accessing elements
let first = numbers[0];  // 10
let second = numbers[1]; // 20
```

### Array Operations

#### Finding Maximum

```fax
fn findMax(arr: []i64): i64 {
    if (len(arr) == 0) {
        return 0;
    }
    
    let max = arr[0];
    let i = 1;
    
    while (i < len(arr)) {
        if (arr[i] > max) {
            max = arr[i];
        }
        i = i + 1;
    }
    
    return max;
}
```

#### Summing Elements

```fax
fn sum(arr: []i64): i64 {
    let total = 0;
    let i = 0;
    
    while (i < len(arr)) {
        total = total + arr[i];
        i = i + 1;
    }
    
    return total;
}
```

#### Reversing an Array

```fax
fn reverse(arr: []i64): []i64 {
    let n = len(arr);
    let i = 0;
    
    while (i < n / 2) {
        let temp = arr[i];
        arr[i] = arr[n - 1 - i];
        arr[n - 1 - i] = temp;
        i = i + 1;
    }
    
    return arr;
}
```

## Structs: Custom Data Types

Structs allow you to group related data together.

### Defining Structs

```fax
struct Point {
    x: i64,
    y: i64
}

struct Person {
    name: str,
    age: i64,
    email: str
}
```

### Creating Struct Instances

```fax
let origin = Point { x: 0, y: 0 };
let target = Point { x: 10, y: 20 };

let alice = Person {
    name: "Alice",
    age: 30,
    email: "alice@example.com"
};
```

### Using Structs in Functions

```fax
fn distance(p1: Point, p2: Point): i64 {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    return dx * dx + dy * dy;  // Squared distance
}

fn greet(person: Person) {
    print("Hello, " + person.name + "!");
    print("You are " + person.age + " years old.");
}
```

## Sorting Algorithms

### Bubble Sort

```fax
fn bubbleSort(arr: []i64): []i64 {
    let n = len(arr);
    let i = 0;
    
    while (i < n - 1) {
        let j = 0;
        while (j < n - i - 1) {
            if (arr[j] > arr[j + 1]) {
                let temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
            j = j + 1;
        }
        i = i + 1;
    }
    
    return arr;
}
```

## Searching Algorithms

### Linear Search

```fax
fn linearSearch(arr: []i64, target: i64): i64 {
    let i = 0;
    while (i < len(arr)) {
        if (arr[i] == target) {
            return i;  // Found at index i
        }
        i = i + 1;
    }
    return -1;  // Not found
}
```

### Binary Search

Binary search requires a sorted array:

```fax
fn binarySearch(arr: []i64, target: i64): i64 {
    let left = 0;
    let right = len(arr) - 1;
    
    while (left <= right) {
        let mid = left + (right - left) / 2;
        
        if (arr[mid] == target) {
            return mid;
        }
        
        if (arr[mid] < target) {
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }
    
    return -1;  // Not found
}
```

## Complete Example

```fax
fn main() {
    // Create array
    let numbers = [64, 34, 25, 12, 22, 11, 90];
    print("Original: " + arrayToString(numbers));
    
    // Sort
    let sorted = bubbleSort(numbers);
    print("Sorted: " + arrayToString(sorted));
    
    // Search
    let target = 25;
    let index = binarySearch(sorted, target);
    if (index >= 0) {
        print("Found " + target + " at index " + index);
    } else {
        print(target + " not found");
    }
    
    // Use structs
    let p1 = Point { x: 0, y: 0 };
    let p2 = Point { x: 3, y: 4 };
    print("Distance squared: " + distance(p1, p2));
}
```

## Performance Considerations

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| Array access | O(1) | Direct indexing |
| Linear search | O(n) | Unsorted data |
| Binary search | O(log n) | Requires sorted data |
| Bubble sort | O(n²) | Simple but slow |

## Challenge Exercises

1. **Implement Insertion Sort**: More efficient than bubble sort for small arrays
2. **Find Median**: Calculate the middle value of a sorted array
3. **Remove Duplicates**: Create a function that returns an array with unique elements
4. **Merge Arrays**: Combine two sorted arrays into one sorted array

## Key Takeaways

- Arrays are fixed-size collections accessed by index
- Structs bundle related data into custom types
- Choose algorithms based on your data characteristics
- Always consider edge cases (empty arrays, etc.)

## Next Steps

- [Building a Todo List](/Fax/tutorials/todo-list/)
- [Error Handling Patterns](/Fax/tutorials/error-handling/)
