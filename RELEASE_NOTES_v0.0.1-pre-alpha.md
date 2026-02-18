# Release Notes: Fax Compiler v0.0.1 pre-alpha

**Release Date:** February 18, 2026

**Tag:** `v0.0.1-pre-alpha`

---

## ⚠️ Pre-Alpha Warning

This is a **pre-alpha release** of the Fax compiler intended for early adopters, contributors, and those interested in exploring the language design. 

### What This Means

- **Incomplete Features**: Not all planned language features are implemented
- **Breaking Changes**: Future releases may include breaking changes without deprecation warnings
- **Performance**: The compiler and runtime are not yet optimized for production use
- **Documentation**: Some documentation may be incomplete or outdated
- **Bugs**: Expect bugs and edge cases that haven't been discovered yet

### Who Should Use This

- Language designers and compiler enthusiasts
- Contributors interested in helping build Fax
- Early adopters who want to experiment with the language
- Researchers studying programming language design

### Who Should NOT Use This

- Production systems
- Projects requiring stability guarantees
- Users who need complete language features
- Anyone unwilling to deal with potential bugs

---

## What's New

### Language Features

#### Type System
- **Primitive Types**: Full support for `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `bool`, `char`, `str`
- **Type Inference**: Automatic type inference for local variables using `let`
- **Type Annotations**: Explicit type annotations on function parameters

#### Functions
```fax
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Lambda expressions
let multiply = fn(x: i32, y: i32) -> i32 { x * y }

// Higher-order functions
fn apply(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}
```

#### Control Flow
```fax
// If expressions (return values)
let max = if a > b { a } else { b }

// Match expressions with pattern matching
match value {
    0 => println("zero"),
    1 => println("one"),
    n if n > 10 => println("large"),
    _ => println("other"),
}

// Loops
while i < 10 {
    i = i + 1
}

loop {
    if done { break }
}
```

#### Data Types
```fax
// Structs
struct Point {
    x: f64,
    y: f64,
}

// Enums (Algebraic Data Types)
enum Result {
    Ok(i32),
    Err(str),
}

// Tuples
let pair = (42, "answer")
let (num, text) = pair
```

### Compiler

#### Lexer (`faxc-lex`)
- Complete token recognition for all language constructs
- Unicode support in identifiers and strings
- Comprehensive error reporting with span information
- Cursor-based efficient lexing

#### Parser (`faxc-par`)
- Recursive descent parser with proper precedence handling
- Full AST construction for all language features
- Pattern parsing for match expressions
- Expression, statement, and declaration parsing

#### Code Generation
- LLVM IR code generation
- Cross-platform support (Linux, macOS, Windows)
- Basic optimization passes

### Garbage Collector (FGC)

The FGC garbage collector provides concurrent, low-latency memory management:

#### Core Features
- **Concurrent Mark-Compact**: Minimizes stop-the-world pauses
- **Generational Collection**: Optimized for object lifetime patterns
- **Precise GC**: Accurate tracking of all object references

#### Allocation
- **TLAB**: Thread-Local Allocation Buffers for fast allocation
- **Bump Pointer**: Efficient sequential allocation
- **Large Objects**: Separate handling for large allocations
- **NUMA-Aware**: Optimized for multi-socket systems

#### Memory Management
- Virtual memory management with memory mapping
- Page and region management
- Alignment guarantees

#### Barriers
- Read barriers for concurrent marking
- Load barriers for precise tracking
- Address space barriers
- Colored pointer support

#### Marking
- Bitmap-based marking for efficiency
- Multi-threaded GC coordination
- Root scanning (stack, globals, registers)
- Object scanning with precise type information

#### Relocation
- Compaction to reduce fragmentation
- Forwarding pointers for object movement
- Copy collection support

#### Runtime Integration
- Finalizer support for cleanup
- Safepoint management for GC coordination
- Automatic initialization

### Infrastructure

#### CI/CD Pipeline
- **Automated Builds**: Build on every push and PR
- **Cross-Platform Testing**: Linux, macOS, Windows
- **Code Coverage**: Track test coverage with reports
- **Security Scanning**: Automated vulnerability scanning
- **Benchmarks**: Performance regression tracking

#### Docker Support
```bash
# Build the image
docker build -t faxc .

# Run the compiler
docker run --rm -v $(pwd):/workspace faxc faxc /workspace/input.fax
```

#### Build Scripts
```bash
# Build the compiler
./scripts/build.sh           # Debug build
./scripts/build.sh --release # Release build

# Run tests
./scripts/test.sh
./scripts/test.sh --release

# Run all checks
./scripts/check.sh

# Verify MSRV
./scripts/verify-msrv.sh
```

---

## Bug Fixes

### Quality Improvements

| ID | Description |
|----|-------------|
| QC-001 | Fixed silent failure pattern in memory operations |
| QC-002 | Added comprehensive memory validation |
| QC-009 | Fixed mutex `unwrap()` calls with proper error handling |

### Other Fixes
- Fixed parser edge cases in nested expressions
- Corrected type inference in complex scenarios
- Fixed GC root scanning issues
- Resolved memory alignment problems
- Improved error messages throughout

---

## Security

### New Security Measures
- **cargo-audit**: Automated scanning for known vulnerabilities in dependencies
- **cargo-deny**: License and dependency policy enforcement
- **GitHub Security Advisories**: Private vulnerability reporting
- **Security Workflow**: Automated security scanning on every PR

### Security Policy
- Established security reporting process
- Defined supported versions
- Created security response guidelines

---

## Getting Started

### Prerequisites
- Rust 1.75 or later
- LLVM 14+ (for code generation)
- Git

### Installation

```bash
# Clone the repository
git clone https://github.com/username/faxc.git
cd faxc

# Build the compiler
cd faxc
cargo build --release

# Verify installation
./target/release/faxc --version
```

### Quick Example

```fax
// examples/01_hello.fax
fn main() {
    println("Hello, Fax!")
}
```

```bash
# Compile and run
./target/release/faxc examples/01_hello.fax
./01_hello
```

---

## Known Issues

### Language
- Pattern matching performance can be improved
- Some error messages could be more helpful
- Limited standard library

### Compiler
- No package manager yet
- Incomplete optimization passes
- Limited IDE support

### Runtime
- GC tuning parameters need documentation
- Some edge cases in concurrent collection

### Documentation
- Some examples may be outdated
- API documentation is incomplete

---

## Contributors

Initial development by the Fax team.

Special thanks to:
- The Rust community for inspiration and best practices
- The LLVM project for the compiler infrastructure
- Early contributors and testers

---

## License

Licensed under either of:

- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))
- **MIT License** ([LICENSE-MIT](LICENSE-MIT))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this release shall be dual licensed as above, without any additional terms or conditions.

---

## Links

- **Repository**: https://github.com/username/faxc
- **Issues**: https://github.com/username/faxc/issues
- **Discussions**: https://github.com/username/faxc/discussions
- **Specification**: https://github.com/username/faxc/blob/main/SPEC.md
- **Contributing Guide**: https://github.com/username/faxc/blob/main/CONTRIBUTING.md

---

## Next Steps

### Planned for v0.0.2
- [ ] Improved error messages
- [ ] More standard library functions
- [ ] Better optimization passes
- [ ] Enhanced documentation
- [ ] Performance improvements

### Future Roadmap
- Module system improvements
- Package manager
- Better IDE support
- Async/await support
- Trait system enhancements
- Macro system

---

*Release notes generated for v0.0.1-pre-alpha*
