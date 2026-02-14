---
title: Troubleshooting
description: Common issues and solutions for Fax
---

# Troubleshooting Guide

This guide helps you resolve common issues when working with Fax.

## Installation Issues

### Error: Command not found

**Problem:**
```
bash: faxt: command not found
```

**Solution:**
Use the full path or create a symlink:

```bash
# Option 1: Use full path
python3 /path/to/faxt/main.py run file.fax

# Option 2: Create alias
alias faxt='python3 /path/to/faxt/main.py'

# Option 3: Add to PATH
export PATH=$PATH:/path/to/fax
```

### Missing Dependencies

**Problem:**
```
Error: Rust not found
Error: Zig not found
Error: GHC not found
```

**Solution:**
Install missing prerequisites:

```bash
# macOS
brew install rust zig ghc node

# Ubuntu/Debian
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
sudo snap install zig --classic
sudo apt-get install ghc nodejs

# Verify installation
python3 faxt/main.py doctor
```

## Compilation Errors

### Syntax Error: Unexpected token

**Problem:**
```
Syntax error at line 5, column 10: Unexpected token '{'
```

**Common Causes:**
- Missing semicolon
- Wrong brace style
- Incorrect indentation

**Solution:**
```fax
// Wrong
fn main() {
    let x = 10
    print(x)
}

// Correct
fn main() {
    let x = 10;
    print(x);
}
```

### Type Mismatch

**Problem:**
```
Type error: Expected i64, found str
```

**Solution:**
Check your types:

```fax
// Wrong
let x: i64 = "hello";  // Error!

// Correct
let x: str = "hello";  // OK
let y: i64 = 42;       // OK
```

### Undefined Variable

**Problem:**
```
Error: Undefined variable 'x'
```

**Solution:**
Declare variables before use:

```fax
// Wrong
print(x);  // Error: x not defined
let x = 10;

// Correct
let x = 10;
print(x);  // OK
```

### Function Not Found

**Problem:**
```
Error: Undefined function 'calculate'
```

**Solution:**
Define functions before calling them:

```fax
// Wrong
fn main() {
    calculate();  // Error: function defined after main
}

fn calculate() { }

// Correct
fn calculate() { }  // Define first

fn main() {
    calculate();  // OK
}
```

## Runtime Errors

### Division by Zero

**Problem:**
```
Runtime error: Division by zero
```

**Solution:**
Add checks before division:

```fax
fn safeDivide(a: i64, b: i64): i64 {
    if (b == 0) {
        print("Warning: Division by zero!");
        return 0;
    }
    return a / b;
}
```

### Array Index Out of Bounds

**Problem:**
```
Runtime error: Array index out of bounds
```

**Solution:**
Check array bounds:

```fax
fn safeGet(arr: []i64, index: i64): i64 {
    if (index < 0 || index >= len(arr)) {
        print("Error: Index " + index + " out of bounds");
        return 0;
    }
    return arr[index];
}
```

### Stack Overflow

**Problem:**
```
Runtime error: Stack overflow
```

**Common Causes:**
- Infinite recursion
- Very deep recursion

**Solution:**
Add base cases and limit recursion depth:

```fax
// Wrong - infinite recursion
fn bad() {
    bad();
}

// Correct
fn factorial(n: i64): i64 {
    if (n <= 1) {  // Base case
        return 1;
    }
    return n * factorial(n - 1);
}
```

## Memory Issues

### Out of Memory

**Problem:**
```
Runtime error: Out of memory
```

**Solution:**
1. Reduce memory usage
2. Trigger garbage collection
3. Check for memory leaks

```fax
// Force GC
if (memoryUsage() > threshold) {
    gc();
}
```

### Memory Leak

**Symptoms:**
- Increasing memory usage over time
- Slower performance

**Solution:**
- Avoid circular references
- Use weak references where appropriate
- Let GC handle cleanup

## Performance Issues

### Slow Compilation

**Solutions:**
1. Use release mode: `faxt build --release`
2. Enable incremental compilation
3. Check for circular imports

### Slow Runtime

**Diagnostics:**
```bash
# Profile the code
faxt bench file.fax
```

**Optimizations:**
- Use `const` for compile-time constants
- Avoid unnecessary allocations
- Use arrays instead of linked lists for random access

## IDE/Editor Issues

### Syntax Highlighting Not Working

**VS Code:**
1. Install the Fax extension
2. Reload window
3. Check file associations

**Vim:**
Add to `.vimrc`:
```vim
autocmd BufRead,BufNewFile *.fax set filetype=fax
```

### Formatter Not Working

**Solution:**
```bash
# Install formatter
faxt install formatter

# Format file
faxt fmt file.fax
```

## Debugging Tips

### Enable Debug Mode

```bash
faxt run --debug file.fax
```

### Add Debug Prints

```fax
fn debug(value: any) {
    print("[DEBUG] " + toString(value));
}

fn main() {
    let x = complexCalculation();
    debug(x);  // Check intermediate value
}
```

### Use Assertions

```fax
fn calculate(a: i64, b: i64): i64 {
    assert(a >= 0, "a must be non-negative");
    assert(b >= 0, "b must be non-negative");
    return a + b;
}
```

## Getting Help

If you're stuck:

1. **Check the docs**: https://luvion1.github.io/Fax/
2. **Search issues**: https://github.com/Luvion1/Fax/issues
3. **Ask on Discord**: Join our community
4. **Read the source**: The compiler is open source!

## Report a Bug

Found a bug? Help us improve:

```bash
# Collect diagnostic info
faxt doctor --verbose > diagnostic.txt

# Create minimal reproduction
faxt minify file.fax > repro.fax
```

Then open an issue on GitHub with:
- Fax version
- Operating system
- Minimal code that reproduces the issue
- Expected vs actual behavior
