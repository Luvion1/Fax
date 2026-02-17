# Fax Compiler

Fax is a modern systems programming language with static typing and garbage collection.

## Features

- **Static Typing**: Type inference with comprehensive checking
- **Garbage Collection**: Low-latency concurrent GC (FGC)
- **Zero-Cost Abstractions**: Compile-time optimizations
- **Memory Safety**: No null pointers, no use-after-free
- **Modern Tooling**: Package manager, debugger, profiler

## Quick Start

```bash
# Build the compiler
./scripts/build.sh

# Compile a Fax program
./target/release/faxc input.fax -o output

# Run tests
./scripts/test.sh
```

## Example

```fax
fn main() {
    let message = "Hello, Fax!";
    print(message);
}
```

## Documentation

- [Language Specification](docs/spec/grammar.md)
- [Architecture Overview](docs/arch/overview.md)
- [Compilation Pipeline](docs/arch/pipeline.md)
- [Technology Stack](docs/arch/technology.md)

## Project Structure

```
faxc/
├── crates/          # Compiler crates
├── tools/           # Developer tools
├── std/             # Standard library
├── tests/           # Test suites
└── docs/            # Documentation
```

## License

MIT OR Apache-2.0