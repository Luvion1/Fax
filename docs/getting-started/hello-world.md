# Hello World in Fax

Your first Fax program!

## Table of Contents

1. [The Program](#the-program)
2. [Step by Step](#step-by-step)
3. [Compile and Run](#compile-and-run)
4. [Variations](#variations)
5. [What's Next](#whats-next)

---

## The Program

Create a file named `hello.fax`:

```fax
fn main() {
    println("Hello, World!")
}
```

---

## Step by Step

Let's break down each part:

### 1. Function Definition

```fax
fn main() {
```

- `fn` - Keyword for defining a function
- `main` - The entry point of every Fax program
- `()` - Parameter list (empty for main)
- `{` - Start of function body

### 2. Print Statement

```fax
    println("Hello, World!")
```

- `println` - Built-in function to print with newline
- `"Hello, World!"` - String literal argument
- `;` - Semicolon (optional for last statement in block)

### 3. Function End

```fax
}
```

- `}` - End of function body

---

## Compile and Run

### Step 1: Create the File

```bash
# Create the file
cat > hello.fax << 'EOF'
fn main() {
    println("Hello, World!")
}
EOF
```

### Step 2: Compile

```bash
# Compile the program
faxc hello.fax
```

This creates an executable named `hello` (or `hello.exe` on Windows).

### Step 3: Run

```bash
# Run the program
./hello
```

### Expected Output

```
Hello, World!
```

---

## Variations

### Multiple Lines

```fax
fn main() {
    println("Hello,")
    println("World!")
}
```

### With Variables

```fax
fn main() {
    let greeting = "Hello"
    let target = "World"
    println(greeting + ", " + target + "!")
}
```

### With Function

```fax
fn greet(name: str) {
    println("Hello, " + name + "!")
}

fn main() {
    greet("World")
    greet("Fax Developer")
}
```

### Formatted Output

```fax
fn main() {
    let name = "Developer"
    let year = 2026
    println("Hello from " + name + " in " + year)
}
```

---

## What's Next

Congratulations! You've written your first Fax program.

### Continue Learning

1. [Quick Tour](quick-tour.md) - Language overview
2. [Variables and Types](../language-guide/basics.md) - Data types
3. [Functions](../language-guide/functions.md) - Writing functions
4. [Control Flow](../language-guide/control-flow.md) - Conditionals and loops

### Practice Exercises

Try modifying the hello world program:

1. Print your name instead of "World"
2. Print a multi-line greeting
3. Create a function that greets multiple people

### Explore Examples

Check out more examples in the [`examples/`](../../faxc/examples/) directory:

- `01_hello.fax` - Basic hello world
- `02_variables.fax` - Variables and types
- `03_functions.fax` - Functions
- `04_match.fax` - Pattern matching

---

## Troubleshooting

### "Command not found: faxc"

Make sure the compiler is installed and in your PATH:

```bash
# Check if faxc is available
which faxc

# If not found, add to PATH
export PATH="$PATH:/path/to/faxc/target/release"
```

### "Cannot open file"

Make sure you're in the correct directory:

```bash
# Check current directory
pwd

# List files
ls -la hello.fax
```

### Compilation Errors

If you see compilation errors, double-check your syntax:

```fax
// Correct
fn main() {
    println("Hello, World!")
}

// Common mistakes:
fn main() {
    println("Hello, World!")  // Missing closing brace
```

---

<div align="center">

**Great start! Keep learning!** 🎉

</div>
