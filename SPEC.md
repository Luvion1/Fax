# Fax Programming Language Specification

## Table of Contents

1. [Overview](#overview)
2. [Design Goals](#design-goals)
3. [Quick Start](#quick-start)
4. [Lexical Specification](#lexical-specification)
5. [Type System](#type-system)
6. [AST Definitions](#ast-definitions)
7. [Language Features](#language-features)
8. [Syntax Reference](#syntax-reference)
9. [Operators](#operators)
10. [Implementation](#implementation)
11. [Compiler Pipeline](#compiler-pipeline)
12. [LLVM IR Generation](#llvm-ir-generation)
13. [Known Limitations](#known-limitations)
14. [Grammar Reference](#grammar-reference)

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

The lexer (`Fax/Lexer.lean`) produces the following token types:

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

```lean
-- From Fax/AST.lean
inductive Type where
  | unit
  | int32 | int64 | float64
  | bool | char | string
  | array (elem : Type) (size : Nat)
  | tuple (elems : List Type)
  | struct (name : String) (fields : List (String × Type))
  | enum (name : String) (variants : List (String × List Type))
  | fun (args : List Type) (ret : Type)
  | inferred
```

---

## 6. AST Definitions

The Abstract Syntax Tree is defined in `Fax/AST.lean`.

### 6.1 Literals

```lean
inductive Literal where
  | int (val : Int)
  | float (val : Float)
  | bool (val : Bool)
  | string (val : String)
  | char (val : Char)
```

### 6.2 Unary Operators

```lean
inductive UnaryOp where
  | neg      -- Unary minus: -x
  | not      -- Logical not: !x
  | bitnot   -- Bitwise not: ~x
```

### 6.3 Binary Operators

```lean
inductive BinOp where
  -- Arithmetic
  | add      -- Addition: +
  | sub      -- Subtraction: -
  | mul      -- Multiplication: *
  | div      -- Division: /
  | mod      -- Modulus: %
  
  -- Logical
  | and      -- Logical AND: &&
  | or       -- Logical OR: ||
  
  -- Comparison
  | eq       -- Equality: ==
  | ne       -- Inequality: !=
  | lt       -- Less than: <
  | le       -- Less than or equal: <=
  | gt       -- Greater than: >
  | ge       -- Greater than or equal: >=
  
  -- Bitwise
  | shl      -- Shift left: <<
  | shr      -- Shift right: >>
  | band     -- Bitwise AND: &
  | bor      -- Bitwise OR: |
  | bxor     -- Bitwise XOR: ^
```

### 6.4 Patterns

```lean
inductive Pat where
  | wild                    -- Wildcard: _
  | lit (l : Literal)       -- Literal pattern: 42, true, "hello"
  | var (name : String)     -- Variable binding: x
  | tuple (pats : List Pat)         -- Tuple pattern: (a, b, c)
  | struct (name : String) (fields : List (String × Pat))  -- Struct pattern
  | enum (name : String) (variant : String) (pats : List Pat)  -- Enum pattern
```

### 6.5 Expressions

```lean
inductive Expr where
  | lit (l : Literal)                           -- Literal value
  | var (name : String)                         -- Variable reference
  | tuple (elems : List Expr)                   -- Tuple literal
  | struct (name : String) (fields : List (String × Expr))  -- Struct literal
  | enum (name : String) (variant : String) (args : List Expr)  -- Enum variant
  | proj (e : Expr) (idx : Nat)                 -- Tuple projection: e.0
  | field (e : Expr) (field : String)           -- Field access: e.field
  | unary (op : UnaryOp) (e : Expr)             -- Unary operation
  | binary (op : BinOp) (e1 e2 : Expr)          -- Binary operation
  | call (fn : String) (args : List Expr)       -- Function call
  | if (cond : Expr) (then : Expr) (else : Expr)  -- If expression
  | match (scrut : Expr) (cases : List (Pat × Expr))  -- Match expression
  | block (stmts : List Stmt) (expr : Expr)     -- Block expression
  | lambda (params : List (String × Type)) (body : Expr)  -- Lambda
  | let (pat : Pat) (value : Expr) (body : Expr)  -- Let binding
```

### 6.6 Statements

```lean
inductive Stmt where
  | decl (mut : Bool) (pat : Pat) (value : Expr)  -- Variable declaration
  | assign (lhs : Expr) (rhs : Expr)               -- Assignment
  | expr (e : Expr)                                -- Expression statement
  | return (e : Expr)                              -- Return statement
  | break                                          -- Break loop
  | continue                                       -- Continue loop
```

### 6.7 Declarations

```lean
inductive Decl where
  | fun (pub : Bool) (name : String) (params : List (String × Type)) (ret : Type) (body : Expr)
  | struct (pub : Bool) (name : String) (fields : List (String × Type))
  | enum (pub : Bool) (name : String) (variants : List (String × List Type))
```

### 6.8 Module

```lean
inductive Module where
  | mk (decls : List Decl)
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
| Bitwise | `&`, `\|`, `^`, `<<`, `>>` | AND, OR, XOR, shift left, shift right |
| Assignment | `=`, `+=`, `-=`, `*=`, `/=`, `%=` | Simple and compound assignment |
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

- **Language**: Lean 4
- **Target**: LLVM IR (can be compiled to native code via llc/clang)
- **Build System**: Lake

### 10.2 Project Structure

```
Fax/
├── lakefile.lean              # Lake build configuration
├── lake-manifest.json         # Dependency manifest
├── SPEC.md                    # This specification
├── Fax/
│   ├── Driver.lean            # Main compiler driver
│   ├── Lexer.lean             # Lexical analyzer (tokenizer)
│   ├── Parser.lean            # Parser (AST builder)
│   ├── AST.lean              # AST node definitions
│   └── Codegen.lean           # LLVM IR code generator
└── .lake/
    └── config/
        └── [anonymous]/
            ├── lakefile.olean
            ├── lakefile.olean.lock
            └── lakefile.olean.trace
```

### 10.3 Dependencies

The project uses:
- **Lean 4** - Programming language and theorem prover
- **LLVM** - Via FFI bindings for code generation

---

## 11. Compiler Pipeline

### 11.1 Compilation Stages

```
Source Code (.fax)
       │
       ▼
┌──────────────────┐
│ 1. Lexical       │  Source → Tokens
│    Analysis      │  Fax/Lexer.lean
└──────────────────┘
       │
       ▼
┌──────────────────┐
│ 2. Parsing      │  Tokens → AST
│                  │  Fax/Parser.lean
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
│    Generation    │  Fax/Codegen.lean
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

### 11.2 Driver Flow (Fax/Driver.lean)

```lean
def compile (source : String) : Except String String :=
  let tokens := Lexer.lex source      -- Stage 1: Lex
  let module ← Parser.parseModule tokens  -- Stage 2: Parse
  let ir := Codegen.Module.toLLVM module   -- Stage 4: Codegen
  return ir
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

The code generator (`Fax/Codegen.lean`) translates AST nodes to LLVM IR:

```lean
-- Type to LLVM type conversion
def Type.toLLVM (ty : AST.Type) : LLVMTypeRef
  -- int32 → i32, float64 → double, bool → i1, etc.

-- Literal to LLVM value
def Literal.toLLVM (l : AST.Literal) (env : CodegenEnv) : LLVMValueRef

-- Expression to LLVM value
def Expr.toLLVM (e : AST.Expr) (env : CodegenEnv) : LLVMValueRef

-- Statement processing
def Stmt.toLLVM (s : AST.Stmt) (env : CodegenEnv) : CodegenEnv

-- Declaration processing
def Decl.toLLVM (d : AST.Decl) (env : CodegenEnv) : CodegenEnv

-- Module to complete LLVM IR
def Module.toLLVM (m : AST.Module) : String
```

### 12.3 Codegen Environment

```lean
structure CodegenEnv where
  module : LLVMModuleRef           -- The LLVM module being built
  builder : LLVMBuilderRef         -- IR builder for inserting instructions
  namedValues : HashMap String LLVMValueRef  -- Variable bindings
  stringConsts : HashMap String LLVMValueRef  -- String literals
```

---

## 13. Known Limitations

### 13.1 Features Not Yet Implemented

| Feature | Status | Location |
|---------|--------|----------|
| **Match expressions** | Parsed but no codegen | AST.lean:51, not in Codegen |
| **Array type** | Defined in AST, no codegen | AST.lean:24 |
| **Field access** | Parsed, no codegen | AST.lean:46 |
| **Assignment statements** | Parsed, no codegen | AST.lean:58 |
| **Enum variants** | Parsed, no codegen | AST.lean:44 |
| **While/Loop statements** | Lexed but not parsed | Lexer.lean:6 |
| **Semantic analysis** | Not implemented | No analyzer module |
| **Type inference** | Partial | Uses `inferred` type |

### 13.2 Incomplete Codegen

The following AST expressions are defined but return placeholder values:

```lean
| .proj e idx => LLVMConstInt (LLVMIntType 32) 0 ...
| .field e field => LLVMConstInt (LLVMIntType 32) 0 ...
| .enum name variant args => LLVMConstInt (LLVMIntType 32) 0 ...
| .match scrut cases => LLVMConstInt (LLVMIntType 32) 0 ...
```

### 13.3 Missing Operators in Codegen

The following BinOp variants are defined but not generated:
- `shl` (shift left)
- `shr` (shift right)
- `band` (bitwise AND)
- `bor` (bitwise OR)
- `bxor` (bitwise XOR)

---

## 14. Grammar Reference

### 14.1 Complete Grammar (EBNF)

```ebnf
program         ::= decl*

decl            ::= fun_decl
                 |  struct_decl
                 |  enum_decl

fun_decl        ::= 'fn' ident '(' param_list ')' ('->' type)? block

param_list      ::= (param (',' param)*)?
param           ::= ident ':' type

struct_decl     ::= 'struct' ident '{' field_list '}'
field_list      ::= (field (',' field)*)?
field           ::= ident ':' type

enum_decl       ::= 'enum' ident '{' variant_list '}'
variant_list    ::= (variant (',' variant)*)?
variant         ::= ident ('(' type_list ')')?

type            ::= 'i8' | 'i16' | 'i32' | 'i64'
                 |  'u8' | 'u16' | 'u32' | 'u64'
                 |  'f32' | 'f64'
                 |  'bool' | 'char' | 'str' | 'unit'
                 |  '[' type ';' number ']'
                 |  '(' type_list ')'
                 |  ident

type_list       ::= type (',' type)*

block           ::= '{' stmt* '}'

stmt            ::= 'let' pattern '=' expr ';'
                 |  'return' expr? ';'
                 |  'break' ';'
                 |  'continue' ';'
                 |  expr ';'

pattern         ::= ident
                 |  '_'
                 |  '(' pattern_list ')'

pattern_list    ::= pattern (',' pattern)*

expr            ::= if_expr
                 |  match_expr
                 |  lambda_expr
                 |  or_expr

if_expr         ::= 'if' expr block ('else' (block | if_expr))?

match_expr      ::= 'match' expr '{' match_arm* '}'
match_arm       ::= pattern '=>' expr (',' | ';')

lambda_expr     ::= 'fn' '(' param_list ')' ('->' type)? expr

or_expr         ::= and_expr ('||' and_expr)*

and_expr        ::= cmp_expr ('&&' cmp_expr)*

cmp_expr        ::= add_expr (('==' | '!=' | '<' | '<=' | '>' | '>=') add_expr)*

add_expr        ::= mul_expr (('+' | '-') mul_expr)*

mul_expr        ::= unary_expr (('*' | '/' | '%') unary_expr)*

unary_expr      ::= ('-' | '!' | '~') unary_expr
                 |  postfix_expr

postfix_expr    ::= primary_expr (('(' arg_list ')') | ('.' ident))*

arg_list        ::= expr (',' expr)*

primary_expr    ::= literal
                 |  ident
                 |  '(' (expr (',' expr)*)? ')'
                 |  '{' block '}'

literal         ::= integer
                 |  float
                 |  string
                 |  char
                 |  'true'
                 |  'false'

integer         ::= [0-9]+
float           ::= [0-9]+ '.' [0-9]+
string          ::= '"' [^"]* '"'
char            ::= '\'' [^']* '\''

ident           ::= [a-zA-Z_] [a-zA-Z0-9_]*
```

---

## Appendix A: Reserved Keywords

```
fn      let     mut     if      else    match
struct  enum    return  true    false   import
pub     mod     use     as      loop    while
for     in      break   continue
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
| 1.0.0 | 2026-02-15 | Initial specification based on implementation analysis |

---

*This specification documents the Fax programming language as implemented in Lean 4, targeting LLVM IR generation.*
