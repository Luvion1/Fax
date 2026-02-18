# Fax Example Programs

This directory contains example Fax programs demonstrating the language features. Each example is self-contained and showcases specific aspects of the Fax programming language.

## Table of Contents

| Example | File | Description |
|---------|------|-------------|
| 1 | [01_hello.fax](01_hello.fax) | Hello World - basic program structure |
| 2 | [02_variables.fax](02_variables.fax) | Variables, mutability, and types |
| 3 | [03_functions.fax](03_functions.fax) | Function declaration, parameters, return values |
| 4 | [04_control_flow.fax](04_control_flow.fax) | if/else, match, while, loop |
| 5 | [05_structs.fax](05_structs.fax) | Struct definition and usage |
| 6 | [06_enums.fax](06_enums.fax) | Enum (ADT) definition and pattern matching |
| 7 | [07_arrays.fax](07_arrays.fax) | Array operations and indexing |
| 8 | [08_tuples.fax](08_tuples.fax) | Tuple usage and destructuring |
| 9 | [09_lambda.fax](09_lambda.fax) | Lambda expressions and higher-order functions |
| 10 | [10_generics.fax](10_generics.fax) | Generic functions and types |

## How to Run

### Prerequisites

- Rust toolchain installed (for building the compiler)
- LLVM installed (for IR generation)

### Building the Compiler

```bash
cd /root/Fax/faxc
cargo build --release
```

### Running Examples

```bash
# Compile a Fax program to LLVM IR
./target/release/faxc examples/01_hello.fax

# Or run with the CLI directly
faxc examples/01_hello.fax
```

### Expected Output

Each example file contains comments showing the expected output. For example, running `01_hello.fax` should output:

```
Hello, Fax!
```

## Language Features Overview

### Basic Features (Examples 1-3)

- **Hello World**: Minimal program with `main()` function
- **Variables**: `let` for immutable, `let mut` for mutable bindings
- **Functions**: `fn` keyword, type annotations, return types with `->`

### Control Flow (Example 4)

- **if/else**: Conditional expressions that return values
- **match**: Pattern matching with guards
- **while/loop**: Iteration with `break` and `continue`

### Data Types (Examples 5-8)

- **Structs**: Product types with named fields
- **Enums**: Sum types (Algebraic Data Types) with variants
- **Arrays**: Fixed-size homogeneous collections `[T; N]`
- **Tuples**: Heterogeneous fixed-size collections `(T, U, V)`

### Advanced Features (Examples 9-10)

- **Lambdas**: Anonymous functions with `fn` syntax
- **Higher-order functions**: Functions that take/return functions
- **Generics**: Type parameters `<T>` for reusable code

## Syntax Quick Reference

### Variable Declaration

```fax
let x = 42           // immutable, type inferred
let mut y = 10       // mutable
let z: f64 = 3.14    // explicit type
```

### Function Definition

```fax
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Lambda Expression

```fax
let double = fn(x: i32) -> i32 { x * 2 }
```

### Struct Definition

```fax
struct Point {
    x: f64,
    y: f64,
}
```

### Enum Definition

```fax
enum Option {
    Some(i32),
    None,
}
```

### Pattern Matching

```fax
match value {
    0 => println("zero"),
    n if n > 0 => println("positive"),
    _ => println("other"),
}
```

## Learning Path

1. Start with **01_hello.fax** to understand basic program structure
2. Move to **02_variables.fax** and **03_functions.fax** for fundamentals
3. Explore **04_control_flow.fax** for branching and iteration
4. Study **05_structs.fax** and **06_enums.fax** for data types
5. Review **07_arrays.fax** and **08_tuples.fax** for collections
6. Advance to **09_lambda.fax** and **10_generics.fax** for powerful abstractions

## Additional Resources

- [Fax Language Specification](../../SPEC.md) - Complete language reference
- [Fax Compiler Source](../) - Implementation details
- [GitHub Repository](https://github.com/your-org/Fax) - Project homepage

## Contributing

When adding new examples:

1. Follow the naming convention: `NN_name.fax` (two-digit number prefix)
2. Include clear comments explaining the feature
3. Add expected output in comments
4. Keep examples focused on a single concept
5. Update this README with the new example

## License

Examples are part of the Fax programming language project. See the main project license for details.
