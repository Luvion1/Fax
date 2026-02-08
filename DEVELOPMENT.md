# FAX Compiler Development Guide

This guide provides comprehensive information for contributing to the FAX compiler project.

## Table of Contents

1. [Architecture](#architecture)
2. [Setting Up Development Environment](#setting-up-development-environment)
3. [Building the Project](#building-the-project)
4. [Development Workflow](#development-workflow)
5. [Code Standards](#code-standards)
6. [Debugging](#debugging)
7. [Common Issues](#common-issues)

## Architecture

FAX is a polyglot compiler where each compilation stage is implemented in the most suitable language:

```
Source (.fax)
    ↓
Lexer (Rust)        → Tokenize with UTF-8 support
    ↓
Parser (Zig)        → Build AST with recursive descent
    ↓
Semantic Analysis (Haskell) → Type check and validate
    ↓
Optimizer (Python)  → Transform and annotate AST
    ↓
Codegen (C++)       → Generate LLVM IR
    ↓
Linker (Zig CC)     → Link with Fgc runtime
    ↓
Binary Executable
```

### Components

| Component | Location | Language | Purpose |
|-----------|----------|----------|---------|
| Hub/Pipeline | `faxc/src/hub/pipeline.ts` | TypeScript | Orchestrates compilation stages |
| Lexer | `faxc/src/components/lexer/` | Rust | Tokenization and scanning |
| Parser | `faxc/src/components/parser/` | Zig | Syntax analysis |
| Semantic Analyzer | `faxc/src/components/sema/` | Haskell | Type checking |
| Optimizer | `faxc/src/components/optimizer/` | Python | AST optimization |
| Codegen | `faxc/src/components/codegen/` | C++ | LLVM IR generation |
| Runtime | `faxc/src/runtime/` | Zig | Fgc garbage collector |

## Setting Up Development Environment

### Prerequisites

- **Node.js** v20+ (for TypeScript/Hub)
- **Rust** (latest stable)
- **Zig** 0.13.0+ (use `zig version`)
- **GHC** 9.6+ (Haskell compiler)
- **Python** 3.10+
- **LLVM** 14+ (for codegen)
- **GCC/Clang** (for C++ compilation)

### Installation on macOS

```bash
# Using Homebrew
brew install node rust zig ghc python@3.10 llvm

# Verify installations
node --version
rustc --version
zig version
ghc --version
python3 --version
```

### Installation on Linux (Ubuntu/Debian)

```bash
# Package managers
sudo apt-get update
sudo apt-get install -y \
    nodejs npm \
    rustc cargo \
    python3 python3-pip \
    ghc cabal-install \
    llvm clang \
    build-essential

# Zig requires manual installation
# Visit https://ziglang.org/download/

# Verify installations
node --version
rustc --version
python3 --version
ghc --version
```

### Cloning and Setup

```bash
git clone https://github.com/anomalyco/fax.git
cd fax
make install

# Or manually:
npm install
cargo fetch
```

## Building the Project

### Debug Build

```bash
# Using Make
make build

# Or manually
npm install
cargo build
zig build
```

### Release Build

```bash
# Using Make
make build-release

# Or manually
cargo build --release
zig build -Doptimize=ReleaseFast
```

### Cleaning Build Artifacts

```bash
make clean
# Or manually
cargo clean && rm -rf zig-cache zig-out && find . -name "*.ll" -delete
```

## Development Workflow

### 1. Making Changes

**Lexer changes (Rust):**
```bash
# Edit faxc/src/components/lexer/lexer.rs
# Rebuild
cargo build
```

**Parser changes (Zig):**
```bash
# Edit faxc/src/components/parser/parser.zig
# Rebuild
zig build
```

**Semantic Analysis changes (Haskell):**
```bash
# Edit faxc/src/components/sema/sema.hs
# Rebuild by recompiling when pipeline calls it
```

**Optimizer changes (Python):**
```bash
# Edit faxc/src/components/optimizer/optimizer.py
# Changes take effect immediately on next run
```

**Codegen changes (C++):**
```bash
# Edit faxc/src/components/codegen/codegen.cpp
# Rebuild by recompiling when pipeline calls it
```

### 2. Testing Changes

```bash
# Test with an example file
make test FILE=example.fax

# Or manually
npm start example.fax

# To compile your own test
echo 'fn main() { print("Test"); }' > mytest.fax
npm start mytest.fax
```

### 3. Debugging

#### View Intermediate Outputs

After compilation, check generated intermediate files:

```bash
# Tokens from lexer (JSON)
cat *.tokens.json

# AST from parser (JSON)
cat *.ast.json

# LLVM IR (see filename.ll)
cat example.ll
```

#### Debug Lexer

```bash
# Run lexer directly with a test file
echo 'fn main() { print(42); }' > test.fax
cargo run --bin lexer -- test.fax
```

#### Debug Parser

```bash
# Manually provide tokens
zig run faxc/src/components/parser/parser.zig -- tokens.json
```

#### Debug Haskell Semantic Analyzer

```bash
# Compile and run
ghc -dynamic faxc/src/components/sema/sema.hs -o sema
./sema ast.json
```

## Code Standards

### Formatting

```bash
# Format all code
make fmt

# Check formatting without changing
make check
```

**Language-specific standards:**

- **TypeScript**: 2-space indent, 100 char line limit
- **Rust**: 4-space indent (enforced by rustfmt)
- **Zig**: 4-space indent (enforced by zig fmt)
- **Haskell**: 2-space indent
- **Python**: PEP 8, 4-space indent

### Documentation

All public interfaces must have documentation:

**TypeScript/JavaScript:**
```typescript
/**
 * Brief description of function.
 * @param param1 - Description
 * @returns Description of return value
 */
function foo(param1: string): void {
  // ...
}
```

**Rust:**
```rust
/// Brief description.
/// 
/// Longer explanation if needed.
/// 
/// # Example
/// ```
/// let x = foo();
/// ```
pub fn foo() -> i32 {
  // ...
}
```

**Haskell:**
```haskell
-- | Brief description
-- Longer explanation
foo :: Int -> String
foo x = ...
```

**Python:**
```python
def foo(param1: str) -> None:
    """Brief description.
    
    Longer explanation if needed.
    
    Args:
        param1: Description
    """
    pass
```

### Error Handling

- Use structured errors with error codes
- Provide actionable error messages
- Include context (line/column numbers)
- Follow Rust error formatting style

## Debugging

### Using Debug Output

Components can produce debug output to stderr:

```bash
# Capture stderr
npm start example.fax 2>debug.log

# View detailed output
cat debug.log
```

### Adding Debug Prints

**Rust:**
```rust
eprintln!("DEBUG: value = {:?}", value);
```

**Zig:**
```zig
std.debug.print("DEBUG: {}\n", .{value});
```

**Haskell:**
```haskell
import Debug.Trace
x `trace` "DEBUG: message"
```

**Python:**
```python
import sys
print("DEBUG: value =", value, file=sys.stderr)
```

## Common Issues

### Issue: "Cannot find zig"

**Solution:** Ensure Zig is installed and in PATH:
```bash
which zig
# If not found, add to PATH or install from https://ziglang.org
```

### Issue: "ghc: command not found"

**Solution:** Install GHC:
```bash
# macOS
brew install ghc

# Linux
sudo apt-get install ghc cabal-install
```

### Issue: Compilation hangs on semantic analysis

**Solution:** Haskell compilation can be slow first time. Subsequent runs are cached.
```bash
# Force rebuild Haskell
rm faxc/src/components/sema/sema_bin
npm start example.fax
```

### Issue: "LLVM version too old"

**Solution:** Upgrade LLVM:
```bash
# macOS
brew upgrade llvm

# Linux
sudo apt-get install llvm-14
```

### Issue: Out of memory during compilation

**Solution:** Increase system memory or use release mode with smaller optimizations:
```bash
# Try with lower optimization level
npm start example.fax -- --release  # Uses O2 instead of O3
```

## Running Tests

```bash
# Run all available test files
for f in *.fax; do npm start "$f"; done

# Run specific test
npm start test_file.fax

# Create simple test
echo 'fn main() { print(123); }' > test_simple.fax
npm start test_simple.fax
```

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for pull request guidelines.

## Additional Resources

- [FAX Language Specification](docs/specification.md)
- [Fgc Architecture](docs/fgc_architecture.md)
- [Compiler Internals](docs/internals/overview.md)
