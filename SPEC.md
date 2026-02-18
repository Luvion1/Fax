# Fax Programming Language Specification

## Table of Contents

1. [Overview](#overview)
2. [Design Goals](#design-goals)
3. [Quick Start](#quick-start)
4. [Lexical Specification](#lexical-specification)
5. [Type System](#type-system)
6. [AST Definitions](#ast-definitions)
7. [Language Features](#language-features)
   - 7.1 Variables and Mutability
   - 7.2 Functions
   - 7.3 Control Flow
   - 7.4 Data Types
   - 7.5 Operators
   - 7.6 Expression Semantics
   - 7.7 Generics
   - 7.8 Traits
   - 7.9 Async/Await
   - 7.10 Error Handling
   - 7.11 Visibility Modifiers
   - 7.12 Constants and Static
8. [Syntax Reference](#syntax-reference)
9. [Operators](#operators)
10. [Implementation](#implementation)
11. [Compiler Pipeline](#compiler-pipeline)
12. [LLVM IR Generation](#llvm-ir-generation)
13. [Known Limitations](#known-limitations)
14. [Grammar Reference](#grammar-reference)
15. [Appendix A: Reserved Keywords](#appendix-a-reserved-keywords)
16. [Appendix B: Example Programs](#appendix-b-example-programs)
17. [Appendix C: Advanced Features](#appendix-c-advanced-features)

---

## 1. Overview

**Fax** is a modern, functional-first programming language with simple syntax inspired by Go and modern features inspired by Rust. It compiles directly to LLVM IR for high-performance native binaries.

Fax combines the simplicity of imperative languages with the expressiveness of functional programming, featuring:
- Static typing with type inference
- First-class functions and lambda expressions
- Algebraic Data Types (ADTs) via enums
- Pattern matching
- Compiles to native code via LLVM

---

## 2. Design Goals

### 2.1 Core Principles

1. **Simple & Clean** - Minimal syntax, maximum readability
   - Low ceremony, no unnecessary keywords
   - Consistent and predictable syntax
   - Readable error messages

2. **Functional-first** - First-class functions, immutability by default
   - Functions as first-class values
   - Immutable data structures by default
   - Higher-order functions
   - Lambda expressions

3. **Modern** - Type inference, pattern matching, algebraic data types
   - Strong static type system
   - Type inference with `let` and function parameters
   - Pattern matching for control flow
   - Algebraic Data Types (ADTs)

4. **Fast** - Compiles to native code via LLVM
   - Direct LLVM IR generation
   - Zero-cost abstractions
   - Native performance

### 2.2 Language Philosophy

- **Explicit where it matters** - Type annotations on function parameters
- **Concise where possible** - Type inference for local variables
- **Safe by default** - Immutable variables unless marked `mut`
- **Pragmatic** - Practical features over theoretical purity

---

## 3. Quick Start

### 3.1 Hello World

```fax
fn main() {
    println("Hello, Fax!")
}
```

### 3.2 Variables

```fax
let x = 42              // immutable (default)
let mut y = 10         // mutable
y = 20
```

### 3.3 Functions

```fax
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Anonymous function (lambda)
let add = fn(a: i32, b: i32) -> i32 { a + b }

// Function with multiple returns using tuples
fn divmod(a: i32, b: i32) -> (i32, i32) {
    (a / b, a % b)
}
```

### 3.4 Control Flow

```fax
// If expression (returns value)
let max = if a > b { a } else { b }

// Match (pattern matching)
match value {
    0 => println("zero"),
    1 => println("one"),
    n if n > 10 => println("large"),
    _ => println("other"),
}

// While loop
let mut i = 0
while i < 5 {
    println(i)
    i = i + 1
}
```

---

## 4. Lexical Specification

### 4.1 Tokens

The lexer (`faxc/lexer/src/lib.rs`) produces the following token types:

#### 4.1.1 Keywords

| Keyword | Description | Usage |
|---------|-------------|-------|
| `fn` | Function declaration | Function definitions |
| `let` | Variable declaration | Immutable bindings |
| `mut` | Mutability modifier | Mutable bindings |
| `if` | Conditional expression | Branching |
| `else` | Alternative branch | If-else blocks |
| `match` | Pattern matching | Pattern matching expression |
| `struct` | Struct declaration | Product types |
| `enum` | Enum declaration | Sum types (ADTs) |
| `return` | Return statement | Function return |
| `true` | Boolean true | Boolean literal |
| `false` | Boolean false | Boolean literal |
| `while` | While loop | Iteration |
| `loop` | Infinite loop | Iteration |
| `break` | Break loop | Loop control |
| `continue` | Continue loop | Loop control |
| `pub` | Public visibility | Module exports |
| `mod` | Module declaration | Module system |
| `use` | Import declaration | Module imports |
| `as` | Type alias | Type renaming |
| `async` | Async function marker | Asynchronous programming |
| `await` | Await async operation | Asynchronous programming |
| `const` | Constant declaration | Compile-time constants |
| `static` | Static variable | Program-lifetime variable |
| `trait` | Trait declaration | Interface definitions |
| `impl` | Implementation block | Trait/struct implementations |
| `dyn` | Dynamic dispatch | Trait objects |
| `where` | Generic constraints | Complex bounds |
| `type` | Type alias | Type renaming |
| `unsafe` | Unsafe block | Unsafe operations |
| `ref` | Reference binding | Pattern matching |
| `self` | Self receiver | Method receiver |
| `Self` | Self type | Current type |
| `super` | Parent module | Module hierarchy |
| `crate` | Current crate | Module hierarchy |
| `for` | For loop keyword | For-in loops |
| `macro_rules` | Macro definition | Macro metaprogramming |

#### 4.1.2 Literals

| Token | Pattern | Example |
|-------|---------|---------|
| `lit_int` | `[0-9]+` | `42`, `0`, `12345` |
| `lit_float` | `[0-9]+\.[0-9]+` | `3.14`, `0.5`, `2.0` |
| `lit_string` | `"[^"]*"` | `"Hello"`, `"World"` |
| `lit_char` | `'[^']'` | `'A'`, `'x'`, `'@'` |

#### 4.1.3 Identifiers

```ebnf
identifier = letter { letter | digit | '_' }
letter     = 'a'..'z' | 'A'..'Z' | '_'
digit      = '0'..'9'
```

Valid: `x`, `foo`, `bar123`, `_private`, `camelCase`
Invalid: `123abc`, `-invalid`

#### 4.1.4 Delimiters

| Token | Symbol | Usage |
|-------|--------|-------|
| `lparen` | `(` | Grouping, function calls, tuples |
| `rparen` | `)` | Close grouping |
| `lbrace` | `{` | Block start |
| `rbrace` | `}` | Block end |
| `lbracket` | `[` | Array/array type start |
| `rbracket` | `]` | Array/array type end |
| `comma` | `,` | Separator |
| `colon` | `:` | Type annotation |
| `semicolon` | `;` | Statement separator |
| `dot` | `.` | Field access |
| `arrow` | `->` | Return type, lambda |
| `pipe` | `\|` | Alternative patterns |
| `underscore` | `_` | Wildcard pattern |
| `arrowfat` | `=>` | Match arm separator |

#### 4.1.5 Operators

See Section 9 for complete operator listing.

---

## 5. Type System

### 5.1 Primitive Types

| Type | Size | Description | LLVM IR Type |
|------|------|-------------|--------------|
| `i8` | 8-bit | Signed integer | `i8` |
| `i16` | 16-bit | Signed integer | `i16` |
| `i32` | 32-bit | Signed integer | `i32` |
| `i64` | 64-bit | Signed integer | `i64` |
| `u8` | 8-bit | Unsigned integer | `i8` |
| `u16` | 16-bit | Unsigned integer | `i16` |
| `u32` | 32-bit | Unsigned integer | `i32` |
| `u64` | 64-bit | Unsigned integer | `i64` |
| `f32` | 32-bit | Float (IEEE 754) | `float` |
| `f64` | 64-bit | Double float | `double` |
| `bool` | 1-bit | Boolean | `i1` |
| `char` | 8-bit | Character | `i8` |
| `str` | pointer | String | `i8*` |
| `unit` | - | Unit type | `i8*` (placeholder) |

### 5.2 Compound Types

#### 5.2.1 Arrays

```fax
let nums: [i32; 3] = [1, 2, 3]
```

Syntax: `[type; size]`

#### 5.2.2 Tuples

```fax
let pair: (i32, str) = (42, "answer")
let (a, b) = pair           // destructuring
```

Syntax: `(type1, type2, ...)`

#### 5.2.3 Structs

```fax
struct Point {
    x: f64,
    y: f64,
}

let p = Point { x: 1.0, y: 2.0 }
let Point { x, y } = p     // destructuring
```

#### 5.2.4 Enums (Algebraic Data Types)

```fax
enum Result {
    Ok(i32),
    Err(str),
}

let r = Result::Ok(42)
match r {
    Result::Ok(v) => println(v),
    Result::Err(e) => println(e),
}
```

#### 5.2.5 Function Types

```fax
fn add(a: i32, b: i32) -> i32 { a + b }
```

Function type syntax: `(param1: type1, param2: type2) -> return_type`

### 5.3 Type Inference

The type `inferred` is used during parsing when no explicit type is provided:

```fax
let x = 42              // type inferred as i32
let y = "hello"         // type inferred as str
```

### 5.4 Type Definitions (AST)

```rust
// From faxc/parser/src/lib.rs
enum Type {
    Unit,
    Int32, Int64, Float64,
    Bool, Char, String,
    Array(Box<Type>, usize),
    Vec<Type>,
    Struct(String, Vec<(String, Type)>),
    Enum(String, Vec<(String, Vec<Type>)>),
    Fun(Vec<Type>, Box<Type>),
    Inferred,
}
```

---

## 6. AST Definitions

The Abstract Syntax Tree is defined in `faxc/parser/src/lib.rs`.

### 6.1 Literals

```rust
enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
}
```

### 6.2 Unary Operators

```rust
enum UnaryOp {
    Neg,      // Unary minus: -x
    Not,      // Logical not: !x
    BitNot,   // Bitwise not: ~x
}
```

### 6.3 Binary Operators

```rust
enum BinOp {
    // Arithmetic
    Add,      // Addition: +
    Sub,      // Subtraction: -
    Mul,      // Multiplication: *
    Div,      // Division: /
    Mod,      // Modulus: %
    
    // Logical
    And,      // Logical AND: &&
    Or,       // Logical OR: ||
    
    // Comparison
    Eq,       // Equality: ==
    Ne,       // Inequality: !=
    Lt,       // Less than: <
    Le,       // Less than or equal: <=
    Gt,       // Greater than: >
    Ge,       // Greater than or equal: >=
    
    // Bitwise
    Shl,      // Shift left: <<
    Shr,      // Shift right: >>
    BitAnd,   // Bitwise AND: &
    BitOr,    // Bitwise OR: |
    BitXor,   // Bitwise XOR: ^
}
```

### 6.4 Patterns

```rust
enum Pat {
    Wild,                    // Wildcard: _
    Lit(Literal),            // Literal pattern: 42, true, "hello"
    Var(String),             // Variable binding: x
    Vec<Pat>,                // Tuple pattern: (a, b, c)
    Struct(String, Vec<(String, Pat)>),  // Struct pattern
    Enum(String, String, Vec<Pat>),      // Enum pattern
}
```

### 6.5 Expressions

```rust
enum Expr {
    Lit(Literal),                           // Literal value
    Var(String),                            // Variable reference
    Tuple(Vec<Expr>),                       // Tuple literal
    Struct(String, Vec<(String, Expr)>),   // Struct literal
    Enum(String, String, Vec<Expr>),       // Enum variant
    Proj(Box<Expr>, usize),                 // Tuple projection: e.0
    Field(Box<Expr>, String),               // Field access: e.field
    Unary(UnaryOp, Box<Expr>),              // Unary operation
    Binary(BinOp, Box<Expr>, Box<Expr>),   // Binary operation
    Call(String, Vec<Expr>),                // Function call
    If(Box<Expr>, Box<Expr>, Box<Expr>),   // If expression
    Match(Box<Expr>, Vec<(Pat, Expr)>),     // Match expression
    Block(Vec<Stmt>, Box<Expr>),            // Block expression
    Lambda(Vec<(String, Type)>, Box<Expr>), // Lambda
    Let(Pat, Box<Expr>, Box<Expr>),         // Let binding
}
```

### 6.6 Statements

```rust
enum Stmt {
    Decl(bool, Pat, Expr),  // Variable declaration (mut, pattern, value)
    Assign(Expr, Expr),     // Assignment
    Expr(Expr),             // Expression statement
    Return(Option<Expr>),   // Return statement
    Break,                  // Break loop
    Continue,               // Continue loop
}
```

### 6.7 Declarations

```rust
struct Decl {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret_type: Type,
    pub body: Vec<Stmt>,
}

struct StructDecl {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

struct EnumDecl {
    pub name: String,
    pub variants: Vec<(String, Vec<Type>)>,
}
```

### 6.8 Module

```rust
struct Module {
    pub declarations: Vec<Decl>,
}
```

---

## 7. Language Features

### 7.1 Variables and Mutability

#### Immutable (Default)

```fax
let x = 42           // immutable
x = 10               // ERROR: cannot reassign
```

#### Mutable

```fax
let mut y = 10       // mutable
y = 20               // OK
```

### 7.2 Functions

#### Basic Function

```fax
fn greet(name: str) -> str {
    "Hello, " + name
}
```

#### Multiple Parameters

```fax
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

#### Multiple Return Values (via Tuples)

```fax
fn divmod(a: i32, b: i32) -> (i32, i32) {
    (a / b, a % b)
}

let (quotient, remainder) = divmod(10, 3)
```

#### Lambda Expressions

```fax
let add = fn(a: i32, b: i32) -> i32 { a + b }
let result = add(1, 2)
```

#### Higher-Order Functions

```fax
fn apply(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fn double(x: i32) -> i32 {
    x * 2
}

let result = apply(double, 5)  // 10
```

### 7.3 Control Flow

#### If Expression

```fax
let max = if a > b { a } else { b }

// Can be used as expression
let kind = if x > 0 { "positive" } else { "non-positive" }
```

#### If-Else Chain

```fax
if x < 0 {
    println("negative")
} else if x == 0 {
    println("zero")
} else {
    println("positive")
}
```

#### Match Expression (Pattern Matching)

```fax
match value {
    0 => println("zero"),
    1 => println("one"),
    n if n > 10 => println("large"),
    _ => println("other"),
}

// Match with destructuring
match point {
    Point { x: 0, y: 0 } => println("origin"),
    Point { x, y } => println("other point"),
}

// Match enum variants
match result {
    Ok(value) => println(value),
    Err(error) => println(error),
}
```

#### While Loop

```fax
let mut i = 0
while i < 5 {
    println(i)
    i = i + 1
}
```

#### Infinite Loop

```fax
loop {
    // infinite loop - must have break
    if done {
        break
    }
}
```

#### Loop Control

```fax
// break - exit loop
let mut i = 0
loop {
    if i >= 5 {
        break
    }
    i = i + 1
}

// continue - skip iteration
let mut sum = 0
let mut i = 0
while i < 10 {
    i = i + 1
    if i % 2 == 0 {
        continue  // skip even numbers
    }
    sum = sum + i
}
```

### 7.4 Data Types

#### Structs (Product Types)

```fax
struct Point {
    x: f64,
    y: f64,
}

struct Person {
    name: str,
    age: i32,
}

// Construction
let p = Point { x: 1.0, y: 2.0 }
let person = Person { name: "Alice", age: 30 }

// Field access
let x = p.x
let name = person.name

// Destructuring
let Point { x, y } = p
```

#### Enums (Sum Types / Algebraic Data Types)

```fax
enum Color {
    Red,
    Green,
    Blue,
    RGB(i32, i32, i32),
}

let c = Color::Red
let rgb = Color::RGB(255, 128, 0)

// Pattern matching
match c {
    Color::Red => println("red"),
    Color::Green => println("green"),
    Color::Blue => println("blue"),
    Color::RGB(r, g, b) => println("RGB"),
}

// Common enum patterns
enum Result {
    Ok(i32),
    Err(str),
}

enum Option {
    Some(i32),
    None,
}
```

#### Tuples

```fax
let pair = (1, "hello")
let first = pair.0       // 1
let second = pair.1      // "hello"

// Destructuring
let (a, b) = pair
let (x, _) = pair        // ignore second element

// Function returning tuple
fn swap(a: i32, b: i32) -> (i32, i32) {
    (b, a)
}
```

#### Arrays

```fax
let nums: [i32; 3] = [1, 2, 3]
let first = nums[0]
```

### 7.5 Operators

#### Arithmetic Operators

```fax
let a = 10 + 5    // 15
let b = 10 - 5    // 5
let c = 10 * 5    // 50
let d = 10 / 5    // 2
let e = 10 % 5    // 0
```

#### Comparison Operators

```fax
let a = 10 == 10   // true
let b = 10 != 5    // true
let c = 10 < 20    // true
let d = 10 <= 10   // true
let e = 20 > 10    // true
let f = 10 >= 10   // true
```

#### Logical Operators

```fax
let a = true && false   // false
let b = true || false   // true
let c = !true          // false
```

#### Bitwise Operators

```fax
let a = 5 & 3          // 1 (0101 & 0011 = 0001)
let b = 5 | 3          // 7 (0101 | 0011 = 0111)
let c = 5 ^ 3          // 6 (0101 ^ 0011 = 0110)
let d = 4 << 1         // 8 (0100 << 1 = 1000)
let e = 8 >> 1         // 4 (1000 >> 1 = 0100)
```

#### Assignment Operators

```fax
let mut x = 10
x = 20
x += 5    // 25
x -= 3    // 22
x *= 2    // 44
x /= 4    // 11
x %= 3    // 2
```

#### Unary Operators

```fax
let a = -5           // unary minus
let b = !false       // logical not
let c = ~3           // bitwise not
```

### 7.6 Expression Semantics

#### Precedence (Low to High)

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 (lowest) | `\|\|` | Left |
| 2 | `&&` | Left |
| 3 | `==`, `!=`, `<`, `<=`, `>`, `>=` | Left |
| 4 | `\|` | Left |
| 5 | `^` | Left |
| 6 | `&` | Left |
| 7 | `<<`, `>>` | Left |
| 8 | `+`, `-` | Left |
| 9 (highest) | `*`, `/`, `%` | Left |

#### Unary operators bind tighter than binary operators

```fax
-5 + 3     // (-5) + 3 = -2
!a && b    // (!a) && b
```

### 7.7 Generics

```fax
fn identity<T>(x: T) -> T {
    x
}

struct Box<T> {
    value: T,
}

enum Option<T> {
    Some(T),
    None,
}

impl<T> Box<T> {
    fn new(value: T) -> Box<T> {
        Box { value }
    }
}
```

### 7.8 Traits

```fax
trait Printable {
    fn print(&self);
}

trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}

impl Add for i32 {
    type Output = i32;
    fn add(self, rhs: i32) -> i32 {
        self + rhs
    }
}
```

### 7.9 Async/Await

```fax
async fn fetch(url: str) -> str {
    let response = http_get(url).await;
    response.body
}

async fn main() {
    let result = await fetch("https://example.com");
    println(result);
}
```

### 7.10 Error Handling

```fax
fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        throw("Division by zero")
    }
    a / b
}

fn safe_divide(a: i32, b: i32) -> Result<i32, str> {
    if b == 0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}
```

### 7.11 Visibility Modifiers

```fax
mod foo {
    pub fn public() {}
    pub(crate) fn crate_visible() {}
    pub(super) fn parent_visible() {}
    fn private() {}
}

pub struct Point {
    pub x: f64,
    pub y: f64,
    hidden: f64,
}
```

### 7.12 Constants and Static

```fax
const MAX: i32 = 100;
const NAME: str = "Fax";

static mut COUNTER: i32 = 0;

fn increment() {
    unsafe {
        COUNTER += 1;
    }
}
```

---

## 8. Syntax Reference

### 8.1 Module

```fax
// File: main.fax
fn main() -> i32 {
    println("Hello, Fax!")
    0
}
```

### 8.2 Function Declaration

```ebnf
fun_decl     ::= 'fn' ident '(' param_list ')' ('->' type)? block
param_list   ::= (param (',' param)*)?
param        ::= ident ':' type
type         ::= primitive_type
             |  tuple_type
             |  array_type
             |  ident
             |  'fn' '(' type_list ')' '->' type
```

### 8.3 Struct Declaration

```ebnf
struct_decl  ::= 'struct' ident '{' field_list '}'
field_list   ::= (field (',' field)*)?
field        ::= ident ':' type
```

### 8.4 Enum Declaration

```ebnf
enum_decl    ::= 'enum' ident '{' variant_list '}'
variant_list ::= (variant (',' variant)*)?
variant      ::= ident ('(' type_list ')')?
```

### 8.5 Statement

```ebnf
stmt         ::= let_stmt
             |  return_stmt
             |  break_stmt
             |  continue_stmt
             |  expr_stmt

let_stmt     ::= 'let' pattern '=' expr
return_stmt  ::= 'return' expr?
break_stmt   ::= 'break'
continue_stmt::= 'continue'
expr_stmt    ::= expr
```

### 8.6 Expression

```ebnf
expr         ::= if_expr
             |  match_expr
             |  lambda_expr
             |  binary_expr

if_expr      ::= 'if' expr block ('else' (block | if_expr))?

match_expr   ::= 'match' expr '{' match_arm* '}'

lambda_expr  ::= 'fn' '(' param_list ')' ('->' type)? expr
```

---

## 9. Operators

### 9.1 All Operators

| Category | Operators | Description |
|----------|-----------|-------------|
| Arithmetic | `+`, `-`, `*`, `/`, `%` | Addition, subtraction, multiplication, division, modulus |
| Comparison | `==`, `!=`, `<`, `<=`, `>`, `>=` | Equality, inequality, less than, etc. |
| Logical | `&&`, `\|\|`, `!` | Logical AND, OR, NOT |
| Bitwise | `&`, `\|`, `^`, `<<`, `>>`, `~` | AND, OR, XOR, shift left, shift right, NOT |
| Assignment | `=`, `+=`, `-=`, `*=`, `/=`, `%=` | Simple and compound assignment |
| Bitwise Assignment | `&=`, `\|=`, `^=`, `<<=`, `>>=` | Bitwise AND, OR, XOR, shift left/right assignment |
| Unary | `-`, `!`, `~` | Negation, logical NOT, bitwise NOT |

### 9.2 Operator Precedence Table

| Precedence | Operator | Description | Associativity |
|------------|----------|-------------|---------------|
| 1 | `\|\|` | Logical OR | Left-to-right |
| 2 | `&&` | Logical AND | Left-to-right |
| 3 | `==`, `!=` | Equality | Left-to-right |
| 3 | `<`, `<=`, `>`, `>=` | Comparison | Left-to-right |
| 4 | `\|` | Bitwise OR | Left-to-right |
| 5 | `^` | Bitwise XOR | Left-to-right |
| 6 | `&` | Bitwise AND | Left-to-right |
| 7 | `<<`, `>>` | Shift | Left-to-right |
| 8 | `+`, `-` | Add/Subtract | Left-to-right |
| 9 | `*`, `/`, `%` | Multiply/Divide/Mod | Left-to-right |
| 10 (highest) | unary `-`, `!`, `~` | Unary operators | Right-to-left |

---

## 10. Implementation

### 10.1 Technology Stack

- **Language**: Rust
- **Target**: LLVM IR (can be compiled to native code via llc/clang)
- **Build System**: Cargo

### 10.2 Project Structure

```
Fax/
├── Cargo.toml                 # Workspace configuration
├── SPEC.md                    # This specification
├── faxc/
│   ├── cli/
│   │   └── src/main.rs        # Main compiler CLI
│   ├── lexer/
│   │   └── src/lib.rs        # Lexical analyzer (tokenizer)
│   ├── parser/
│   │   └── src/lib.rs        # Parser (AST builder)
│   ├── codegen/
│   │   └── src/lib.rs        # LLVM IR code generator (using Inkwell)
│   └── runtime/
│       └── src/lib.rs        # Runtime library with ZGC garbage collector
└── examples/                  # Example Fax programs
```

### 10.3 Dependencies

The project uses:
- **Rust** - Programming language
- **Inkwell** - Rust bindings for LLVM
- **Clap** - CLI argument parsing
- **libc** - Low-level memory management for runtime

---

## 11. Compiler Pipeline

### 11.1 Compilation Stages

```
Source Code (.fax)
        │
        ▼
┌──────────────────┐
│ 1. Lexical       │  Source → Tokens
│    Analysis      │  faxc/lexer/src/lib.rs
└──────────────────┘
        │
        ▼
┌──────────────────┐
│ 2. Parsing      │  Tokens → AST
│                  │  faxc/parser/src/lib.rs
└──────────────────┘
        │
        ▼
┌──────────────────┐
│ 3. Semantic      │  (Not yet implemented)
│    Analysis      │  Type checking, scope resolution
└──────────────────┘
        │
        ▼
┌──────────────────┐
│ 4. Code          │  AST → LLVM IR
│    Generation    │  faxc/codegen/src/lib.rs
└──────────────────┘
        │
        ▼
    LLVM IR (.ll)
        │
        ▼
┌──────────────────┐
│ 5. Native        │  LLVM IR → Executable
│    Compilation   │  via llc + clang
└──────────────────┘
        │
        ▼
    Executable
```

### 11.2 Driver Flow (faxc/cli/src/main.rs)

```rust
fn main() {
    // Stage 1: Lex - Tokenize source code
    let tokens = lexer.tokenize(source);
    
    // Stage 2: Parse - Build AST from tokens
    let ast = parser.parse(tokens).expect("Parse error");
    
    // Stage 4: Codegen - Generate LLVM IR
    let ir = codegen.generate_ir(ast).expect("Codegen error");
    
    println!("{}", ir);
}
```

---

## 12. LLVM IR Generation

### 12.1 Type Mapping

| Fax Type | LLVM IR Type |
|----------|--------------|
| `unit` | `i8*` (placeholder) |
| `i8` | `i8` |
| `i16` | `i16` |
| `i32` | `i32` |
| `i64` | `i64` |
| `f32` | `float` |
| `f64` | `double` |
| `bool` | `i1` |
| `char` | `i8` |
| `str` | `i8*` |
| `array<T; N>` | `[N x T]` |
| `tuple(T1, T2, ...)` | `{ T1, T2, ... }` |

### 12.2 Codegen Implementation

The code generator (`faxc/codegen/src/lib.rs`) translates AST nodes to LLVM IR using Inkwell:

```rust
// Type to LLVM type conversion
impl FaxCodegen {
    fn compile_type(&self, ty: &Type) -> BasicTypeEnum<'ctx>
    
    // Literal to LLVM value
    fn compile_literal(&self, lit: &Literal) -> BasicValueEnum<'ctx>
    
    // Expression to LLVM value
    fn compile_expr(&self, expr: &Expr) -> Result<BasicValueEnum<'ctx>, String>
    
    // Statement processing
    fn compile_stmt(&self, stmt: &Stmt) -> Result<(), String>
    
    // Declaration processing
    fn compile_decl(&self, decl: &Decl) -> Result<FunctionValue<'ctx>, String>
    
    // Module to complete LLVM IR
    fn compile_ast(&self, ast: AST) -> Result<String, String>
}
```

### 12.3 Codegen Environment

```rust
pub struct FaxCodegen<'ctx> {
    pub context: &'ctx Context,        // LLVM context
    pub module: Module<'ctx>,           // The LLVM module being built
    pub builder: Builder<'ctx>,         // IR builder for inserting instructions
}
```

---

## 13. Known Limitations

### 13.1 Features Not Yet Implemented

| Feature | Status | Location |
|---------|--------|----------|
| **Match expressions** | Parsing partial | parser/src/lib.rs |
| **Array type** | Defined in lexer, no codegen | lexer/src/lib.rs |
| **Field access** | Tokenized, no codegen | lexer/src/lib.rs |
| **Assignment statements** | Partial support | parser/src/lib.rs |
| **Enum variants** | Parsed, limited codegen | parser/src/lib.rs |
| **Struct declarations** | Not implemented | - |
| **While/Loop statements** | Tokenized, limited parsing | lexer/src/lib.rs |
| **Semantic analysis** | Not implemented | - |
| **Type inference** | Partial | Uses `inferred` type |
| **Generics** | Grammar defined | SPEC.md |
| **Traits** | Grammar defined | SPEC.md |
| **Async/Await** | Grammar defined | SPEC.md |
| **Error handling** | Grammar defined | SPEC.md |
| **Visibility modifiers** | Grammar defined | SPEC.md |
| **Const/Static** | Grammar defined | SPEC.md |
| **Attributes/Derive** | Grammar defined | SPEC.md |

### 13.2 Incomplete Codegen

The code generator currently only supports basic integer literals:

```rust
fn compile_expr(&self, expr: Expr) -> Result<BasicValueEnum<'ctx>, String> {
    match expr {
        Expr::Lit(Literal::Int(v)) => {
            Ok(self.context.i32_type().const_int(v as u64, false).into())
        }
        _ => Err("Expression not supported in Codegen".to_string()),
    }
}
```

### 13.3 Missing Operators in Codegen

The following operations are not yet generated:
- Floating point operations
- String operations
- Boolean operations
- All binary operators except basic integer literals

### 13.4 Grammar Status

| Feature | Grammar Status | Implementation Status |
|---------|---------------|---------------------|
| Basic syntax | ✅ Complete | ✅ Implemented |
| Generics | ✅ Complete | ❌ Not implemented |
| Traits | ✅ Complete | ❌ Not implemented |
| Async/Await | ✅ Complete | ❌ Not implemented |
| Error handling | ✅ Complete | ❌ Not implemented |
| Visibility | ✅ Complete | ❌ Not implemented |
| Attributes | ✅ Complete | ❌ Not implemented |

---

## 14. Grammar Reference

### 14.1 Complete Grammar (EBNF)

```ebnf
program         ::= item*

item            ::= fun_decl
                 |  struct_decl
                 |  enum_decl
                 |  trait_decl
                 |  impl_decl
                 |  const_decl
                 |  static_decl
                 |  use_decl
                 |  mod_decl

fun_decl        ::= 'fn' ident generics? '(' param_list ')' ('->' type)? where_clause? body
                 |  'async' 'fn' ident generics? '(' param_list ')' ('->' type)? where_clause? body

generics        ::= '<' generic_param (',' generic_param)* '>' 
generic_param   ::= ident (':' type_bound)?
where_clause    ::= 'where' constraint (',' constraint)*

param_list      ::= (param (',' param)*)?
param           ::= ident ':' type

body            ::= block
                 |  ';'   (for extern declarations)

struct_decl     ::= 'struct' ident generics? ('{' field_list '}')?
field_list      ::= (field (',' field)*)?
field           ::= ident ':' type
                 |  ident ':' type '=' expr

enum_decl       ::= 'enum' ident generics? '{' variant_list '}'
variant_list    ::= (variant (',' variant)*)?
variant         ::= ident ('(' type_list ')')?

trait_decl      ::= 'trait' ident generics? ('where' constraint_list)? '{' trait_item* '}'
trait_item      ::= fun_sig
                 |  const_sig

impl_decl       ::= 'impl' generics? type ('for' type)? '{' impl_item* '}'
impl_item       ::= fun_def
                 |  const_def

const_decl      ::= 'const' ident ':' type '=' expr ';'
static_decl     ::= 'static' 'mut'? ident ':' type '=' expr ';'

use_decl        ::= 'use' path ('as' ident)? ';'
mod_decl        ::= 'mod' ident (';' | '{' item* '}')

type            ::= 'i8' | 'i16' | 'i32' | 'i64'
                 |  'u8' | 'u16' | 'u32' | 'u64'
                 |  'f32' | 'f64'
                 |  'bool' | 'char' | 'str' | 'unit'
                 |  '[' type ';' number ']'
                 |  '(' type_list ')'
                 |  ident generics?
                 |  '&' type
                 |  'dyn' type_bound
                 |  'fn' '(' type_list ')' ('->' type)?

type_bound      ::= ident ('+' ident)*
type_list       ::= type (',' type)*
constraint      ::= type '<' type
                 |  type ':' type_bound

block           ::= '{' stmt* '}'

stmt            ::= 'let' pattern ('=' expr)? ';'
                 |  'let' 'mut' pattern ('=' expr)? ';'
                 |  'return' expr? ';'
                 |  'break' expr? ';'
                 |  'continue' ';'
                 |  'while' expr block
                 |  'loop' block
                 |  'for' pattern 'in' expr block
                 |  expr ';'

pattern         ::= ident
                 |  '_'
                 |  literal
                 |  '(' pattern (',' pattern)* ')'
                 |  ident '@' pattern

pattern_list    ::= pattern (',' pattern)*

expr            ::= if_expr
                 |  match_expr
                 |  lambda_expr
                 |  await_expr
                 |  try_expr
                 |  or_expr

if_expr         ::= 'if' expr block ('else' (block | if_expr))?

match_expr      ::= 'match' expr '{' match_arm* '}'
match_arm       ::= pattern ('if' guard)? '=>' expr

guard           ::= expr

lambda_expr     ::= 'fn' generics? '(' param_list ')' ('->' type)? block

await_expr      ::= 'await' expr

try_expr        ::= 'try' expr

or_expr         ::= and_expr ('||' and_expr)*

and_expr        ::= cmp_expr ('&&' cmp_expr)*

cmp_expr        ::= add_expr (('==' | '!=' | '<' | '<=' | '>' | '>=') add_expr)*

add_expr        ::= mul_expr (('+' | '-') mul_expr)*

mul_expr        ::= unary_expr (('*' | '/' | '%') unary_expr)*

unary_expr      ::= ('-' | '!' | '~') unary_expr
                 |  postfix_expr

postfix_expr    ::= primary_expr (('(' arg_list ')') | ('.' ident | '?'))*

arg_list        ::= expr (',' expr)*

primary_expr    ::= literal
                 |  path ('::' ident)*
                 |  '(' (expr (',' expr)*)? ')'
                 |  '[' (expr (',' expr)*)? ']'
                 |  '{' struct_field (',' struct_field)* '}'
                 |  '[' expr ';' expr ']'  (array repetition)

path            ::= ident ('::' ident)*

struct_field    ::= ident ':' expr
                 |  ident

literal         ::= integer
                 |  float
                 |  string
                 |  char
                 |  'true'
                 |  'false'

integer         ::= [0-9]+
                 |  0x[0-9a-fA-F]+
                 |  0b[01]+
                 |  0o[0-7]+

float           ::= [0-9]+ '.' [0-9]+ (('e' | 'E') ('+' | '-') [0-9]+)?
                 |  [0-9]+ ('e' | 'E') ('+' | '-') [0-9]+

string          ::= '"' [^"]* '"'
                 |  'r' '#* '"' [^"]* '"' '#*'

char            ::= '\'' [^']* '\''

ident           ::= [a-zA-Z_] [a-zA-Z0-9_]*
```

---

## Appendix A: Reserved Keywords

```
fn      let     mut     if      else    match
struct  enum    return  true    false   pub
mod     use     as      loop    while   for
break   continue  async   await   const
static  trait   impl    dyn     where   type
unsafe  ref     self    Self    super   crate
macro_rules
```

## Appendix C: Advanced Features

### C.1 Generics

```fax
fn identity<T>(x: T) -> T {
    x
}

fn pair<T, U>(a: T, b: U) -> (T, U) {
    (a, b)
}

struct Container<T> {
    value: T,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### C.2 Traits

```fax
trait Printable {
    fn print(&self);
    
    fn debug(&self) -> str {
        // Default implementation
        "debug"
    }
}

trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}

impl Printable for i32 {
    fn print(&self) {
        println(*self)
    }
}
```

### C.3 Async/Await

```fax
async fn fetch_data(url: str) -> str {
    let response = http_get(url).await;
    response.body
}

async fn main() {
    let data = await fetch_data("https://example.com");
    println(data);
}
```

### C.4 Error Handling

```fax
fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        throw("Division by zero")
    }
    a / b
}

fn safe_divide(a: i32, b: i32) -> i32? {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

fn handle_error() {
    let result = try divide(10, 0);
    match result {
        Ok(value) => println(value),
        Err(e) => println(e),
    }
}
```

### C.5 Visibility and Modules

```fax
mod math {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    
    pub(crate) fn internal() {}
    
    pub(super) fn parent_accessible() {}
    
    mod inner {
        pub(super) fn to_parent() {}
    }
}

use math::add;
use math::{sub, mul};

mod external {
    extern "C" {
        fn printf(format: *const u8, ...);
    }
}
```

### C.6 Constants and Static

```fax
const MAX_SIZE: i32 = 100;
const DEFAULT_NAME: str = "Guest";

static mut COUNTER: i32 = 0;

fn increment() {
    unsafe {
        COUNTER = COUNTER + 1;
    }
}
```

### C.7 Attributes and Derive

```fax
#[derive(Clone, Debug, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Default)]
struct Config {
    host: str = "localhost",
    port: u16 = 8080,
}

#[inline]
fn hot_path() {}

#[cold]
fn error_path() {}

#[cfg(target_os = "linux")]
fn linux_only() {}
```

## Appendix B: Example Programs

### B.1 Hello World

```fax
fn main() {
    println("Hello, Fax!")
}
```

### B.2 Fibonacci

```fax
fn fib(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fn main() {
    let mut i = 0
    while i < 10 {
        println(fib(i))
        i = i + 1
    }
}
```

### B.3 Struct and Enum

```fax
struct Point {
    x: f64,
    y: f64,
}

enum Color {
    Red,
    Green,
    Blue,
    RGB(i32, i32, i32),
}

fn main() {
    let p = Point { x: 1.0, y: 2.0 }
    let c = Color::RGB(255, 128, 0)
    
    match c {
        Color::Red => println("red"),
        Color::Green => println("green"),
        Color::Blue => println("blue"),
        Color::RGB(r, g, b) => println("RGB"),
    }
}
```

### B.4 Higher-Order Functions

```fax
fn apply(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fn double(x: i32) -> i32 {
    x * 2
}

fn main() {
    let result = apply(double, 5)
    println(result)
}
```

---

## Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.0.1 | 2026-02-15 | Initial specification based on implementation analysis |

---

*This specification documents the Fax programming language as implemented in Rust using Inkwell for LLVM IR generation.*
