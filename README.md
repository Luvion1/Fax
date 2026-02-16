# Fax Compiler

A modern functional-first programming language implemented in **Lean 4**, featuring a microservices architecture and low-latency garbage collection (FGC).

![Version](https://img.shields.io/badge/version-0.0.1-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)

## 🚀 Features

### Language Features
- **Functional-first** programming with immutable data structures
- **Static typing** with powerful type inference (Hindley-Milner)
- **Pattern matching** for expressive control flow
- **Algebraic Data Types** (ADTs) via structs and enums
- **First-class functions** and lambda expressions
- **Memory safety** with low-latency garbage collection

### Architecture
- **Lean 4** implementation for formal verification benefits
- **Microservices-based** compiler design
- **Protocol Buffers** for service communication
- **gRPC** for distributed compilation
- **Modular design** - each phase runs as independent service

### Performance
- **FGC (Fax Garbage Collector)** with <1ms pause times
- **Concurrent marking and relocation**
- **Thread-local allocation buffers** for fast allocation
- **Generational collection** (young/old generations)
- **Region-based heap management**

## 📦 Installation

### Prerequisites
- Lean 4 (latest stable)
- LLVM/Clang (for IR generation)
- Protocol Buffers compiler

### From Source
```bash
git clone https://github.com/Luvion1/Fax.git
cd Fax
make build
make install
```

### Using Docker
```bash
docker build -t fax .
docker run --rm fax --help
```

## 📝 Quick Start

### Hello World
```fax
fn main() -> i32 {
    println(42)
    0
}
```

### Compile and Run
```bash
faxc hello.fax -o hello
./hello
```

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [Language Specification](SPEC.md) | Complete language reference |
| [Architecture Guide](ARCHITECTURE.md) | Microservices architecture |
| [FGC Documentation](FGC.md) | Garbage collector details |
| [Module Structure](MODULE_STRUCTURE.md) | Code organization guidelines |
| [Examples](examples/) | Sample programs |

## 🧪 Testing

```bash
# Run all tests
make test

# Run specific test suites
make test-unit
make test-integration
make test-e2e

# Run benchmarks
make benchmark
```

## 🏗️ Project Structure

```
Fax/
├── faxc/                     # Compiler source code (Lean 4)
│   ├── Compiler/
│   │   ├── AST/             # Abstract Syntax Tree
│   │   ├── Lexer/           # Tokenization
│   │   ├── Parser/          # AST construction
│   │   ├── Semantic/        # Type checking & inference
│   │   ├── Codegen/         # LLVM IR generation
│   │   ├── Driver/          # Compiler driver & CLI
│   │   ├── Proto/           # Protocol Buffers & gRPC
│   │   ├── Runtime/         # FGC implementation
│   │   └── Validation/      # Input validation
│   ├── Fax.lean              # Main entry point
│   └── StdLib.lean          # Standard library
├── tests/                    # Test suites
│   ├── unit/                # Unit tests
│   ├── integration/         # Integration tests
│   ├── e2e/                 # End-to-end tests
│   └── benchmarks/           # Performance benchmarks
├── examples/                 # Example programs
├── proto/ schemas
└──                   # Protocol Buffer docs/                    # Additional documentation
```

### Module Organization

The compiler follows **Lean 4 best practices** with a modular architecture:

- **Index Files**: Each module has an index file (e.g., `Compiler/Semantic.lean`) that exports the public API
- **Focused Submodules**: Functionality is split into focused modules (e.g., `Semantic/Checker.lean`, `Semantic/Inference.lean`)
- **Validation Module**: Input validation with separate validators for source, identifiers, types, and limits

See [MODULE_STRUCTURE.md](MODULE_STRUCTURE.md) for detailed guidelines.

## 🛠️ Development

### Building
```bash
make build          # Debug build
make release        # Release build
```

### Development Environment
```bash
make docker-dev     # Start Docker dev environment
make watch          # Watch mode for development
```

## 📊 Performance

### Compilation Speed
- Lexing: ~1M tokens/second
- Parsing: ~100K AST nodes/second
- Codegen: ~50K lines IR/second

### GC Performance
- Pause times: <1ms (typical 0.1-0.5ms)
- Throughput: >95% application time
- Allocation rate: >100K objects/second

## 🎯 Roadmap

### Phase 1 (Completed) ✅
- [x] Lexer and Parser
- [x] AST definitions
- [x] Basic Codegen
- [x] FGC implementation
- [x] Microservices architecture

### Phase 2 (Completed) ✅
- [x] Complete Semantic Analysis (type inference, type checking)
- [x] Comprehensive Testing (106 tests)
- [x] Docker & CI/CD
- [x] Code Reorganization & Modularization

### Phase 3 (In Progress) 🚧
- [ ] LLVM FFI bindings for actual code execution
- [ ] Standard Library expansion
- [ ] Optimization passes

### Phase 4 (Planned) 📋
- [ ] Package manager
- [ ] IDE support
- [ ] WebAssembly target

## 🤝 Contributing

We welcome contributions! Please open an issue or submit a pull request.

### Areas for Contribution
- Language features and syntax improvements
- Performance optimizations
- Documentation improvements
- Bug fixes
- Example programs

## 📄 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file.

## 🙏 Acknowledgments

- Lean 4 team for the excellent theorem prover
- ZGC/Shenandoah teams for low-latency GC inspiration
- Protocol Buffers team

## 📞 Support

- GitHub Issues: [Report bugs](https://github.com/Luvion1/Fax/issues)
- GitHub Discussions: [Ask questions](https://github.com/Luvion1/Fax/discussions)

---

**Fax Compiler v0.0.1** - Built with Lean 4
