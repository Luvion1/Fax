---
title: Built-in Functions
description: Reference for all built-in functions in Fax
---

# Built-in Functions

Fax provides several built-in functions for common operations.

## Output Functions

### print

Outputs a value to the console with automatic newline.

```fax
print("Hello, World!");
print(42);
print(true);
```

**Parameters:**
- `value`: Any type (i64, str, bool, etc.)

**Returns:** `void`

## Type Functions

### len

Returns the length of an array or string.

```fax
let arr = [1, 2, 3, 4, 5];
let length = len(arr);  // 5

let s = "hello";
let strLen = len(s);    // 5
```

**Parameters:**
- `value`: Array or string

**Returns:** `i64` - The length

### typeof

Returns the type of a value as a string.

```fax
let x = 10;
let t = typeof(x);  // "i64"
```

## Math Functions

### sqrt

Calculates the square root of a number.

```fax
let x = sqrt(16);  // 4.0
let y = sqrt(2);   // 1.414...
```

**Parameters:**
- `n`: i64 or f64

**Returns:** f64

### abs

Returns the absolute value of a number.

```fax
let x = abs(-10);  // 10
let y = abs(10);   // 10
```

**Parameters:**
- `n`: i64

**Returns:** i64

### pow

Raises a number to a power.

```fax
let x = pow(2, 8);   // 256
let y = pow(10, 3);  // 1000
```

**Parameters:**
- `base`: i64
- `exp`: i64

**Returns:** i64

### min

Returns the smaller of two values.

```fax
let x = min(10, 5);   // 5
let y = min(-3, -5);  // -5
```

### max

Returns the larger of two values.

```fax
let x = max(10, 5);   // 10
let y = max(-3, -5);  // -3
```

## String Functions

### concat

Concatenates two strings.

```fax
let s = concat("Hello, ", "World!");  // "Hello, World!"
```

**Note:** The `+` operator can also be used for string concatenation.

### substring

Extracts a portion of a string.

```fax
let s = "Hello, World!";
let sub = substring(s, 0, 5);  // "Hello"
```

**Parameters:**
- `str`: str - The source string
- `start`: i64 - Starting index (inclusive)
- `end`: i64 - Ending index (exclusive)

**Returns:** str

### split

Splits a string into an array.

```fax
let s = "a,b,c";
let parts = split(s, ",");  // ["a", "b", "c"]
```

### trim

Removes whitespace from both ends of a string.

```fax
let s = "  hello  ";
let t = trim(s);  // "hello"
```

### toString

Converts a value to a string.

```fax
let n = 42;
let s = toString(n);  // "42"
```

## Array Functions

### push

Adds an element to the end of an array.

```fax
let arr = [1, 2, 3];
push(arr, 4);  // [1, 2, 3, 4]
```

### pop

Removes and returns the last element.

```fax
let arr = [1, 2, 3];
let last = pop(arr);  // 3, arr is now [1, 2]
```

### slice

Returns a portion of an array.

```fax
let arr = [1, 2, 3, 4, 5];
let sub = slice(arr, 1, 4);  // [2, 3, 4]
```

### sort

Sorts an array in place.

```fax
let arr = [3, 1, 4, 1, 5];
sort(arr);  // [1, 1, 3, 4, 5]
```

### reverse

Reverses an array in place.

```fax
let arr = [1, 2, 3];
reverse(arr);  // [3, 2, 1]
```

## GC Functions

### gc

Triggers garbage collection.

```fax
gc();  // Force garbage collection
```

### gcStats

Returns GC statistics.

```fax
let stats = gcStats();
print("Heap size: " + stats.heapSize);
```

## System Functions

### exit

Exits the program with a status code.

```fax
exit(0);  // Success
exit(1);  // Error
```

### time

Returns the current timestamp in milliseconds.

```fax
let now = time();
```

### sleep

Pauses execution for a specified time.

```fax
sleep(1000);  // Sleep for 1 second
```

## Complete Example

```fax
fn main() {
    // Math functions
    print("sqrt(16) = " + sqrt(16));
    print("abs(-5) = " + abs(-5));
    print("pow(2, 10) = " + pow(2, 10));
    
    // String functions
    let s = "Hello, World!";
    print("Length: " + len(s));
    print("Substring: " + substring(s, 0, 5));
    
    // Array functions
    let arr = [3, 1, 4, 1, 5];
    print("Original: " + toString(arr));
    sort(arr);
    print("Sorted: " + toString(arr));
    print("Reversed: " + toString(reverse(arr)));
}
```
