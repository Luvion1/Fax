# Getting Started with Fax

Welcome to the Fax programming language! This guide will help you get up and running quickly.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Quick Tour](#quick-tour)
4. [Hello World](#hello-world)
5. [Next Steps](#next-steps)

---

## Prerequisites

Before installing Fax, ensure you have the following:

### Required
- **Rust 1.75 or later** ([install via rustup](https://rustup.rs))
- **Git** for cloning the repository

### Optional (for development)
- **LLVM 14+** (for code generation)
- **A text editor or IDE** (VS Code, RustRover, etc.)

### Verify Prerequisites

```bash
# Check Rust version (must be >= 1.75.0)
rustc --version

# Check Git
git --version
```

---

## Installation

### Option 1: Build from Source (Recommended for Development)

```bash
# Clone the repository
git clone https://github.com/Luvion1/Fax.git
cd Fax

# Build the compiler (debug mode)
cd faxc
cargo build

# Build for release (optimized)
cargo build --release

# Add to PATH (optional)
export PATH="$PWD/target/release:$PATH"
```

### Option 2: Using Docker

```bash
# Pull the Docker image
docker pull ghcr.io/luvion1/fax:latest

# Run the compiler
docker run --rm -v $(pwd):/workspace ghcr.io/luvion1/fax:latest /workspace/hello.fax
```

### Option 3: Pre-built Binaries

Download pre-built binaries from the [Releases page](https://github.com/Luvion1/Fax/releases).

| Platform | Download |
|----------|----------|
| Linux x86_64 | `faxc-linux-x86_64.tar.gz` |
| macOS x86_64 | `faxc-macos-x86_64.tar.gz` |
| Windows x86_64 | `faxc-windows-x86_64.zip` |

---

## Quick Tour

Fax is a modern, functional-first programming language that compiles to LLVM IR. Here's what makes it special:

### Key Features

- **Functional-First Design** - First-class functions, immutability by default
- **Static Typing with Inference** - Strong types without verbose annotations
- **Algebraic Data Types** - Expressive data modeling with enums and structs
- **Pattern Matching** - Powerful `match` expressions
- **Native Performance** - Compiles to optimized machine code via LLVM

### Language Comparison

| Feature | Fax | Rust | Go |
|---------|-----|------|-----|
| Type Inference | ✅ | ✅ | ❌ |
| Pattern Matching | ✅ | ✅ | ❌ |
| Garbage Collection | ✅ | ❌ | ✅ |
| Functional-First | ✅ | Partial | ❌ |
| Compilation Speed | Fast | Moderate | Fast |

---

## Hello World

Let's write your first Fax program!

### Create the File

Create a file named `hello.fax`:

```fax
fn main() {
    println("Hello, World!")
}
```

### Compile and Run

```bash
# Compile the program
faxc hello.fax

# Run the compiled binary
./hello
```

**Output:**
```
Hello, World!
```

### Understanding the Code

```fax
fn main() {           // Define the main function
    println("...")    // Print to stdout
}                     // End of function
```

- `fn` - Keyword for function definition
- `main` - Entry point of the program
- `println` - Built-in function for printing

---

## Next Steps

Now that you have Fax installed and running, explore these resources:

### Learn the Language
- [Quick Tour](quick-tour.md) - Language overview
- [Type System](../language-guide/types.md) - Understanding Fax types
- [Functions](../language-guide/functions.md) - Functions and lambdas
- [Pattern Matching](../language-guide/pattern-matching.md) - Match expressions

### Dive Deeper
- [Language Specification](../../SPEC.md) - Complete grammar reference
- [Examples](../../faxc/examples/) - Sample Fax programs
- [Architecture](../compiler/architecture.md) - How the compiler works

### Get Involved
- [Contributing Guide](../../CONTRIBUTING.md) - How to contribute
- [GitHub Discussions](https://github.com/Luvion1/Fax/discussions) - Ask questions
- [GitHub Issues](https://github.com/Luvion1/Fax/issues) - Report bugs

---

## Troubleshooting

### Common Issues

#### "Command not found: faxc"

Make sure the compiler is in your PATH:

```bash
export PATH="$PATH:/path/to/faxc/target/release"
```

#### "Rust version too old"

Update Rust to the latest version:

```bash
rustup update
```

#### "LLVM not found"

Install LLVM for your platform:

```bash
# Ubuntu/Debian
sudo apt-get install llvm

# macOS
brew install llvm

# Windows
# Download from https://llvm.org/releases/
```

---

<div align="center">

**Happy Coding with Fax!** 🚀

</div>
