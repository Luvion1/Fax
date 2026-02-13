---
sidebar_position: 2
---

# Quick Start

## Build

```bash
# Clone and build
git clone https://github.com/Luvion1/Fax.git
cd Fax
npm install
make build
```

## Your First Program

Create `hello.fax`:

```fax
fn main() {
    let message = "Hello, Fax!";
    print(message);
}
```

## Run

```bash
# Using Python
python3 faxt/main.py run hello.fax

# Or create symlink
chmod +x faxt/main.py
ln -sf $(pwd)/faxt/main.py /usr/local/bin/faxt
faxt run hello.fax
```

## CLI Commands

```bash
faxt new <name>      # Create project
faxt run <file>      # Compile & run
faxt build <file>    # Compile to target/
faxt check <file>    # Type check only
faxt test            # Run tests
faxt repl            # Interactive REPL
```

## Examples

### Variables

```fax
fn main() {
    let x = 10;           // immutable
    let mut y = 0;       // mutable
    y = y + 1;
    print(y);
}
```

### Control Flow

```fax
fn main() {
    if x > 5 {
        print("big");
    } elif x > 3 {
        print("medium");
    } else {
        print("small");
    }
    
    // While
    let mut i = 0;
    while i < 5 {
        print(i);
        i = i + 1;
    }
    
    // For (range)
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
    print(add(5, 3));
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
}
```

### Pattern Matching

```fax
fn main() {
    let x = 2;
    match x {
        1 => print("one"),
        2 => print("two"),
        default => print("other"),
    }
}
```

## Next Steps

- [Language Guide](/docs/language/basics)
- [Architecture](/docs/architecture/overview)
