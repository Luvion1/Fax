# Getting Started with Fax

## Installation

### Prerequisites

Install the following tools:

- **Rust** (for Lexer, Optimizer)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Zig** (for Parser, Runtime)
  ```bash
  wget https://ziglang.org/download/0.14.1/zig-linux-x86_64-0.14.1.tar.xz
  tar -xf zig-linux-x86_64-0.14.1.tar.xz
  sudo mv zig-linux-x86_64-0.14.1 /opt/zig
  export PATH=$PATH:/opt/zig
  ```

- **GHC** (for Semantic Analyzer)
  ```bash
  sudo apt-get install ghc cabal-install
  ```

- **Node.js** (for CLI tooling)
  ```bash
  nvm install 22
  ```

- **C++ Compiler** with LLVM
  ```bash
  sudo apt-get install llvm-dev clang
  ```

### Clone and Build

```bash
git clone https://github.com/Luvion1/Fax.git
cd Fax

# Install Node dependencies
npm install

# Build all components
make build

# Or build individually
cd faxc/packages/lexer && cargo build --release
cd faxc/packages/parser && zig build
cd faxc/packages/sema && ghc -o sema src/Main.hs
cd faxc/packages/optimizer && cargo build --release
cd faxc/packages/codegen && mkdir build && cd build && cmake .. && make
cd faxc/packages/runtime && zig build
```

## Quick Start

### Your First Program

Create a file `hello.fax`:

```fax
fn main() {
    let name = "Fax";
    print("Hello, " + name + "!");
}
```

### Running Your Program

Using the CLI tool:

```bash
# Make CLI executable
chmod +x faxt/main.py

# Run directly
python3 faxt/main.py run hello.fax

# Or create a symlink
ln -sf $(pwd)/faxt/main.py /usr/local/bin/faxt
faxt run hello.fax
```

### CLI Commands

```bash
faxt new <project_name>   # Create new project
faxt run <file>           # Compile and run
faxt build <file>         # Compile to target/
faxt check <file>         # Type check only
faxt test                 # Run tests
faxt repl                 # Interactive REPL
```

## Examples

### Variables

```fax
fn main() {
    // Immutable variable
    let x = 10;
    
    // Mutable variable
    let mut count = 0;
    count = count + 1;
    
    print(count);
}
```

### Control Flow

```fax
fn main() {
    let x = 10;
    
    if x > 5 {
        print("x is greater than 5");
    } elif x > 3 {
        print("x is greater than 3");
    } else {
        print("x is small");
    }
    
    // While loop
    let mut i = 0;
    while i < 5 {
        print(i);
        i = i + 1;
    }
    
    // For loop (range)
    for j in 0..10 {
        print(j);
    }
}
```

### Functions

```fax
fn add(a: i64, b: i64) -> i64 {
    return a + b;
}

fn main() {
    let result = add(5, 3);
    print(result);
}
```

### Structs

```fax
struct Point {
    x: i64,
    y: i64,
}

fn main() {
    let p = Point { x: 10, y: 20 };
    print(p.x);
    print(p.y);
}
```

### Pattern Matching

```fax
fn main() {
    let value = 2;
    
    match value {
        1 => print("one"),
        2 => print("two"),
        default => print("other"),
    }
}
```

## Troubleshooting

### "command not found: faxt"

Make sure the CLI is executable:
```bash
chmod +x faxt/main.py
```

Or use Python directly:
```bash
python3 faxt/main.py run hello.fax
```

### Build Errors

Make sure all prerequisites are installed. See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed setup instructions.

## Next Steps

- Read the [Language Guide](language.md) for more syntax
- Check [Architecture](architecture.md) to understand the compiler
- Explore [Examples](../examples/) in the repository
