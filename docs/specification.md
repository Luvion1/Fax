# Dokumentasi Bahasa Pemrograman Fax

## Daftar Isi
1. [Tentang Fax](#tentang-fax)
2. [Fitur Utama](#fitur-utama)
3. [Sintaks Dasar](#sintaks-dasar)
4. [Tipe Data](#tipe-data)
5. [Struktur Kontrol](#struktur-kontrol)
6. [Fungsi](#fungsi)
7. [Module dan Namespace](#module-dan-namespace)
8. [Trait dan Generics](#trait-dan-generics)
9. [Pattern Matching](#pattern-matching)
10. [Error Handling](#error-handling)
11. [Memory Management](#memory-management)
12. [Standard Library](#standard-library)
13. [Toolchain](#toolchain)

## Tentang Fax

Fax adalah bahasa pemrograman modern yang dirancang untuk memberikan kombinasi sempurna antara kemudahan penggunaan, keamanan tinggi, dan performa tinggi. Bahasa ini menggabungkan elemen terbaik dari berbagai paradigma pemrograman dan mengadopsi sintaks modern yang bersih dan ekspresif.

### Filosofi Desain
- **Maintainability**: Sintaks yang bersih dan mudah dipahami
- **Safety**: Type safety dan memory safety tanpa overhead runtime
- **Performance**: Compiled ke native code dengan performa tinggi
- **Flexibility**: Mendukung paradigma deklaratif, prosedural, dan fungsional

## Fitur Utama

### 1. Sintaks Modern dan Ekspresif
```fax
// Komentar satu baris
/* Komentar
   multi-baris */

// Fungsi dasar
fn main() {
    let greeting = "Hello, World!"
    io.println(greeting)
}

// Fungsi dengan type annotation
fn add(a: Int, b: Int): Int {
    a + b
}
```

### 2. Static Typing dengan Type Inference
```fax
// Type inference
let x = 42        // x: Int
let name = "Bob"  // name: String

// Explicit type annotation
let pi: Float = 3.14159
let is_valid: Bool = true
```

### 3. Compiled dengan GC
```fax
// Tidak perlu khawatir tentang ownership
fn example() {
    let data = vec![1, 2, 3, 4, 5]  // disimpan di heap
    let shared = data               // tidak ada ownership transfer
    // GC akan membersihkan ketika tidak digunakan
}
```

## Sintaks Dasar

### Variabel dan Konstanta
```fax
// Immutable by default
let x = 42
let name = "Alice"

// Mutable variable
var counter = 0
counter += 1

// Constant
const MAX_SIZE: Int = 100
```

### Komentar
```fax
// Komentar satu baris
/* Komentar
   multi-baris */
/// Dokumentasi fungsi
```

### Operator
```fax
// Arithmetic
let sum = a + b
let diff = a - b
let prod = a * b
let quot = a / b
let rem = a % b

// Comparison
let equal = a == b
let greater = a > b
let less_eq = a <= b

// Logical
let and_result = a && b
let or_result = a || b
let not_result = !a

// Assignment
var x = 5
x += 3  // x = x + 3
x *= 2  // x = x * 2
```

## Tipe Data

### Tipe Primitif
```fax
// Integer types
let byte_val: Byte = 255
let int_val: Int = 42
let long_val: Long = 1000000

// Floating point
let float_val: Float = 3.14
let double_val: Double = 3.14159265359

// Boolean
let is_true: Bool = true
let is_false: Bool = false

// Character and String
let char_val: Char = 'A'
let str_val: String = "Hello"
```

### Collection Types
```fax
// Array (fixed size)
let arr: [Int; 3] = [1, 2, 3]

// Vector (dynamic size)
let vec: Vec<Int> = vec![1, 2, 3, 4, 5]

// HashMap
let map: HashMap<String, Int> = HashMap::new()
map.insert("key".to_string(), 42)

// Tuple
let pair: (Int, String) = (1, "one")
let (num, text) = pair  // destructuring
```

### Custom Types
```fax
// Struct
struct Person {
    name: String,
    age: Int,
}

// Enum
enum Color {
    Red,
    Green,
    Blue,
}

// Enum with data
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Using custom types
fn main() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    }
    
    let success = Result::Ok(42)
    let failure = Result::Err("Something went wrong".to_string())
}
```

## Struktur Kontrol

### Conditional Statements
```fax
// If-else
if x > 0 {
    io.println("Positive")
} else if x < 0 {
    io.println("Negative")
} else {
    io.println("Zero")
}

// Ternary-like expression
let abs_x = if x >= 0 { x } else { -x }

// Match expression
match color {
    Color::Red => io.println("Red"),
    Color::Green => io.println("Green"),
    Color::Blue => io.println("Blue"),
}
```

### Loop Constructs
```fax
// For loop
for i in 0..10 {
    io.println(i)
}

// For-in with collections
for item in vec {
    io.println(item)
}

// While loop
while condition {
    // do something
    if break_condition {
        break
    }
}

// Infinite loop
loop {
    // do something
    if exit_condition {
        break
    }
}
```

## Fungsi

### Deklarasi Fungsi
```fax
// Basic function
fn greet(name: String) {
    io.println(format!("Hello, {}!", name))
}

// Function with return type
fn add(a: Int, b: Int): Int {
    a + b
}

// Function with multiple return values (tuple)
fn divide_with_remainder(dividend: Int, divisor: Int) -> (Int, Int) {
    (dividend / divisor, dividend % divisor)
}

// Anonymous function (lambda)
let square = |x| x * x
let numbers = vec![1, 2, 3, 4, 5]
let squares = numbers.map(square)
```

### Higher-Order Functions
```fax
// Function that takes another function
fn apply_twice<F>(f: F, x: Int) -> Int where F: Fn(Int) -> Int {
    f(f(x))
}

// Closure
fn create_multiplier(factor: Int) -> impl Fn(Int) -> Int {
    move |x| x * factor
}

let double = create_multiplier(2)
let result = double(5)  // 10
```

## Module dan Namespace

### Sistem Module
```fax
// File: math.fax
pub fn factorial(n: Int): Int {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}

pub struct Calculator {
    pub total: Int,
}

impl Calculator {
    pub fn new() -> Self {
        Calculator { total: 0 }
    }
    
    pub fn add(&mut self, value: Int) {
        self.total += value
    }
}

// File: main.fax
use std::io
use math::{factorial, Calculator}  // Import specific items
use std::collections::Vec          // Import with full path

fn main() {
    let result = factorial(5)
    io.println(format!("5! = {}", result))
    
    let mut calc = Calculator::new()
    calc.add(10)
    io.println(format!("Total: {}", calc.total))
}
```

### Namespace dengan `::`
```fax
// Nested modules
use graphics::render::Renderer
use graphics::shapes::{Circle, Rectangle}

fn main() {
    let mut renderer = Renderer::new()
    let circle = Circle::new(0.0, 0.0, 5.0)
    let rect = Rectangle::new(0.0, 0.0, 10.0, 10.0)
    
    renderer.draw_shape(circle)
    renderer.draw_shape(rect)
}
```

## Trait dan Generics

### Trait Definition
```fax
// Define a trait
trait Drawable {
    fn draw(&self)
    fn area(&self) -> Float
}

// Implement trait for a type
impl Drawable for Circle {
    fn draw(&self) {
        io.println("Drawing a circle")
    }
    
    fn area(&self) -> Float {
        3.14159 * self.radius * self.radius
    }
}

// Generic function with trait bounds
fn print_area<T: Drawable>(shape: &T) {
    io.println(format!("Area: {}", shape.area()))
}
```

### Generics
```fax
// Generic struct
struct Container<T> {
    value: T,
}

// Generic function
fn identity<T>(x: T) -> T {
    x
}

// Multiple generic parameters
fn pair<A, B>(first: A, second: B) -> (A, B) {
    (first, second)
}

// Where clause for complex bounds
fn complex_operation<T, U>(t: T, u: U) -> bool
where
    T: Clone + Debug,
    U: PartialEq + From<T>,
{
    let cloned_t = t.clone()
    U::from(cloned_t) == u
}
```

## Pattern Matching

### Match Expression
```fax
// Basic match
match value {
    0 => io.println("Zero"),
    1..=10 => io.println("Small number"),
    _ => io.println("Large number"),
}

// Match with binding
match option_val {
    Some(x) => io.println(format!("Got value: {}", x)),
    None => io.println("No value"),
}

// Match with guards
match point {
    Point { x, y } if x == y => io.println("Diagonal point"),
    Point { x: 0, y } => io.println(format!("On Y axis: {}", y)),
    Point { x, y: 0 } => io.println(format!("On X axis: {}", x)),
    Point { x, y } => io.println(format!("Point({}, {})", x, y)),
}

// Destructuring tuples
match (x, y) {
    (0, 0) => io.println("Origin"),
    (0, _) => io.println("On Y axis"),
    (_, 0) => io.println("On X axis"),
    _ => io.println("Somewhere else"),
}
```

### If Let
```fax
// Simplified pattern matching for single case
if let Some(value) = optional_value {
    io.println(format!("Got: {}", value))
}

// Multiple patterns
if let Ok(result) = computation_result {
    io.println(format!("Success: {}", result))
} else if let Err(error) = computation_result {
    io.println(format!("Error: {}", error))
}
```

## Error Handling

### Result Type
```fax
// Define a function that can fail
fn divide(a: Float, b: Float) -> Result<Float, String> {
    if b == 0.0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// Handle Result with match
fn handle_division() {
    match divide(10.0, 2.0) {
        Ok(result) => io.println(format!("Result: {}", result)),
        Err(error) => io.println(format!("Error: {}", error)),
    }
}

// Handle Result with if let
fn handle_division_simple() {
    if let Ok(result) = divide(10.0, 2.0) {
        io.println(format!("Result: {}", result))
    }
}

// Propagate errors with ?
fn calculate_something() -> Result<Float, String> {
    let a = divide(10.0, 2.0)?  // Will return early if Err
    let b = divide(a, 5.0)?     // Will return early if Err
    Ok(b)
}
```

### Option Type
```fax
// Working with potentially missing values
fn find_user(id: Int) -> Option<User> {
    // Implementation that might return None
    if id > 0 {
        Some(User::new(id))
    } else {
        None
    }
}

// Safe unwrapping
fn process_user(id: Int) {
    match find_user(id) {
        Some(user) => io.println(format!("Found user: {}", user.name())),
        None => io.println("User not found"),
    }
}

// Using methods on Option
fn get_username_or_default(id: Int) -> String {
    find_user(id)
        .map(|user| user.name())
        .unwrap_or_else(|| "Anonymous".to_string())
}
```

## Memory Management

### Garbage Collection
```fax
// Memory managed automatically by GC
fn memory_example() {
    let data = vec![1, 2, 3, 4, 5]  // allocated on heap
    let shared_ref = data.clone()    // shallow copy, reference counted
    
    // Both variables can be used safely
    process_data(data)
    process_data(shared_ref)
    // GC will clean up when no references remain
}

fn process_data(vec: Vec<Int>) {
    for item in vec {
        io.println(item)
    }
    // vec goes out of scope, GC handles cleanup
}

// Circular references handled by tracing GC
struct Node {
    value: Int,
    children: Vec<Node>,
    parent: Option<*Node>,  // weak reference to prevent cycles
}
```

## Standard Library

### Collections
```fax
use std::collections::{Vec, HashMap, HashSet}

// Dynamic array
let mut numbers = Vec::new()
numbers.push(1)
numbers.push(2)
numbers.extend([3, 4, 5])

// Hash map
let mut scores = HashMap::new()
scores.insert("Alice".to_string(), 95)
scores.insert("Bob".to_string(), 87)

// Hash set
let mut unique_items = HashSet::new()
unique_items.insert(1)
unique_items.insert(2)
unique_items.insert(1)  // duplicate ignored
```

### I/O Operations
```fax
use std::io

// Console output
io.println("Hello, World!")
io.print("Enter your name: ")

// File operations
use std::fs
let content = fs::read_to_string("file.txt")?
fs::write("output.txt", "Hello, World!")?
```

### String Operations
```fax
use std::string

let mut text = "Hello".to_string()
text.push(' ')
text.push_str("World!")

let formatted = format!("Value: {}", 42)
let split_parts = text.split_whitespace().collect::<Vec<_>>()
```

## Toolchain

### Compiler (faxc)
```bash
# Compile single file
faxc main.fax

# Compile with optimizations
faxc --release main.fax

# Compile with debug info
faxc --debug main.fax

# Generate documentation
faxc --doc main.fax
```

### Package Manager
```bash
# Initialize new project
fax init my_project

# Add dependency
fax add serde

# Build project
fax build

# Run project
fax run

# Run tests
fax test

# Format code
fax fmt

# Check code
fax check
```

### REPL
```bash
# Start interactive session
fax repl

# In REPL:
>>> let x = 42
>>> x + 8
50
>>> fn square(n) { n * n }
>>> square(5)
25
```

## System Requirements

### Minimum System Requirements
- Operating System: Linux, macOS, or Windows
- CPU: Dual-core processor (2 GHz or faster recommended)
- RAM: 2 GB minimum, 4 GB recommended
- Disk Space: 1 GB free space for basic installation
- Network: Required for downloading dependencies

### Language-Specific Requirements

#### Rust Backend
- Rust compiler (rustc) version 1.60+
- Cargo package manager
- Additional disk space: 500MB - 1GB
- C compiler (gcc/clang) for certain dependencies

#### C++ Integration
- C++ compiler (GCC 7+, Clang 6+, or MSVC 2019+)
- CMake version 3.10+
- Additional disk space: 200MB - 1GB

#### Python Integration
- Python 3.7+ with pip
- Additional disk space: 50MB - 200MB
- Virtual environment support recommended

#### JavaScript Integration
- Node.js version 14+ with npm
- Additional disk space: 50MB - 200MB

#### Zig Integration
- Zig compiler version 0.10+
- Additional disk space: 100MB - 200MB

#### Haskell Integration
- GHC (Glasgow Haskell Compiler) version 8.0+
- Cabal or Stack build tool
- Additional disk space: 1-2 GB

## Multi-Language Integration

### Overview
Fax supports seamless integration with multiple programming languages, allowing developers to leverage existing codebases and libraries. The multi-language support enables:

- Calling Fax functions from other languages
- Using libraries from other languages in Fax programs
- Creating hybrid applications combining multiple languages
- Gradual migration from existing codebases

### Integration Methods

#### Foreign Function Interface (FFI)
Fax provides robust FFI support for direct integration with C-compatible libraries:

```fax
// Example of FFI declaration
extern "C" {
    fn printf(format: *const Char, ...) -> Int;
    fn malloc(size: USize) -> *mut Void;
}
```

#### Language Bindings
Each supported language has dedicated binding mechanisms:

##### C/C++ Bindings
- Direct linking with C/C++ libraries
- Compatible with C ABI
- Support for common data structures

##### Python Bindings
- Python extension modules
- Automatic type conversion
- Access to Python standard library

##### JavaScript Bindings
- Node.js addon support
- V8 engine integration
- Asynchronous operation support

##### Rust Bindings
- Native integration with Rust ecosystem
- Zero-cost abstractions
- Memory safety guarantees

##### Zig Bindings
- Direct compilation compatibility
- Minimal runtime overhead
- Cross-language optimization

##### Haskell Bindings
- Foreign function interface
- Lazy evaluation integration
- Type system compatibility

### Building Multi-Language Projects

The Fax build system handles multi-language projects seamlessly:

```bash
# Build project with all language integrations
fax build --all-targets

# Build specific language targets
fax build --target cpp
fax build --target python
fax build --target js

# Generate bindings for specific languages
fax gen-bindings --lang python
fax gen-bindings --lang js
```

## Contoh Program Lengkap

```fax
use std::io
use std::collections::HashMap

// Define a trait
trait Greetable {
    fn greet(&self) -> String
}

// Define a struct
struct Person {
    name: String,
    age: Int,
}

// Implement trait for struct
impl Greetable for Person {
    fn greet(&self) -> String {
        format!("Hello, I'm {} and I'm {} years old", self.name, self.age)
    }
}

// Generic function
fn introduce<T: Greetable>(entity: &T) {
    io.println(entity.greet())
}

// Main function
fn main() {
    // Create a person
    let alice = Person {
        name: "Alice".to_string(),
        age: 30,
    }
    
    // Introduce the person
    introduce(&alice)
    
    // Use collections
    let mut scores = HashMap::new()
    scores.insert(alice.name.clone(), 95)
    
    // Pattern matching
    match scores.get("Alice") {
        Some(score) => io.println(format!("Alice's score: {}", score)),
        None => io.println("Score not found"),
    }
    
    // Functional programming
    let numbers = vec![1, 2, 3, 4, 5]
    let doubled = numbers
        .iter()
        .map(|&x| x * 2)
        .filter(|&x| x > 4)
        .collect::<Vec<_>>()
    
    io.println(format!("Doubled numbers > 4: {:?}", doubled))
}
```
