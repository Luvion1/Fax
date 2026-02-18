# Changelog

All notable changes to the Fax compiler project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial pre-alpha release infrastructure

### Changed
- N/A

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A

---

## [0.0.1-pre-alpha] - 2026-02-18

### ⚠️ Pre-Alpha Warning

This is the initial pre-alpha release of the Fax compiler. It is intended for:
- Early adopters and experimenters
- Contributors interested in the project
- Learning and exploration

**Not recommended for production use.** Expect:
- Incomplete language features
- Breaking changes in future releases
- Potential performance issues
- Limited documentation

### Added

#### Language Features
- **Type System**
  - Basic primitive types: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `bool`, `char`, `str`
  - Type inference for local variables
  - Type annotations on function parameters

- **Functions**
  - Function declarations with parameters and return types
  - Lambda expressions (anonymous functions)
  - Higher-order functions
  - Multiple return values via tuples

- **Control Flow**
  - `if`/`else` expressions (return values)
  - `match` expressions with pattern matching
  - `while` loops
  - `loop` (infinite loops)
  - `break` and `continue` statements

- **Data Types**
  - Structs (product types)
  - Enums (sum types / algebraic data types)
  - Tuples
  - Arrays

- **Operators**
  - Arithmetic: `+`, `-`, `*`, `/`, `%`
  - Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
  - Logical: `&&`, `||`, `!`
  - Bitwise: `&`, `|`, `^`, `<<`, `>>`, `~`
  - Assignment: `=`, `+=`, `-=`, `*=`, `/=`, `%=`

- **Pattern Matching**
  - Literal patterns
  - Variable binding patterns
  - Tuple destructuring
  - Struct destructuring
  - Enum variant matching
  - Guard clauses

#### Compiler
- **Lexer** (`faxc-lex`)
  - Token recognition for all language constructs
  - Unicode support in identifiers
  - Comprehensive error reporting

- **Parser** (`faxc-par`)
  - Recursive descent parser
  - AST construction
  - Expression parsing with correct precedence
  - Pattern parsing

- **Code Generation**
  - LLVM IR code generation
  - Cross-platform support (Linux, macOS, Windows)
  - Basic optimizations

#### Garbage Collector (FGC)
- **Core GC**
  - Concurrent mark-compact algorithm
  - Stop-the-world minimization
  - Precise garbage collection

- **Allocation**
  - TLAB (Thread-Local Allocation Buffers)
  - Bump pointer allocation
  - Large object allocation

- **Memory Management**
  - NUMA-aware allocation
  - Virtual memory management
  - Memory mapping

- **Barriers**
  - Read barriers
  - Load barriers
  - Address space barriers
  - Colored pointers

- **Marking**
  - Bitmap-based marking
  - GC thread coordination
  - Root scanning
  - Stack scanning
  - Object scanning

- **Relocation**
  - Compaction
  - Forwarding pointers
  - Copy collection

- **Runtime**
  - Finalizer support
  - Safepoint management
  - Initialization

#### Infrastructure
- **CI/CD Pipeline**
  - Automated builds on push
  - Cross-platform testing (Linux, macOS, Windows)
  - Code coverage tracking
  - Security scanning
  - Benchmark workflows

- **Docker Support**
  - Multi-stage builds
  - Minimal runtime image
  - Development environment

- **Scripts**
  - `build.sh` - Build the compiler
  - `test.sh` - Run test suite
  - `check.sh` - Run all checks
  - `verify-msrv.sh` - Verify MSRV compatibility

- **Documentation**
  - Language specification (SPEC.md)
  - Architecture documentation
  - API documentation for FGC
  - Contributing guidelines
  - Security policy
  - Code of conduct

### Changed

- N/A (Initial release)

### Deprecated

- N/A (Initial release)

### Removed

- N/A (Initial release)

### Fixed

#### Quality Improvements (QC)
- **QC-001**: Fixed silent failure pattern in memory operations
- **QC-002**: Added comprehensive memory validation
- **QC-009**: Fixed mutex `unwrap()` calls with proper error handling
- Added security scanning workflows with `cargo-deny`
- Improved error messages throughout the codebase

#### Bug Fixes
- Fixed various parser edge cases
- Corrected type inference in nested expressions
- Fixed GC root scanning issues
- Resolved memory alignment problems

### Security

- Implemented `cargo-audit` for dependency vulnerability scanning
- Added `cargo-deny` for license and dependency policy enforcement
- Configured GitHub security advisories
- Established security reporting process

### Known Issues

- Pattern matching performance can be improved
- Some error messages could be more helpful
- Limited standard library
- No package manager yet
- Incomplete optimization passes

### Contributors

Initial development by the Fax team.

### License

Licensed under either of:
- Apache License, Version 2.0 ([]())
- MIT license ([LICENSE](LICENSE))

at your option.

---

## Legend

- **Added**: New features or functionality
- **Changed**: Changes to existing functionality
- **Deprecated**: Features that will be removed soon
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Security improvements and fixes

[Unreleased]: https://github.com/username/faxc/compare/v0.0.1-pre-alpha...HEAD
[0.0.1-pre-alpha]: https://github.com/username/faxc/releases/tag/v0.0.1-pre-alpha
