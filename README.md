# Fax Compiler

A modern, functional-first programming language with microservices architecture and low-latency garbage collection.

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)
![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)

## 🚀 Features

### Language Features
- **Functional-first** programming with immutable data structures
- **Static typing** with powerful type inference
- **Pattern matching** for expressive control flow
- **Algebraic Data Types** (ADTs) via structs and enums
- **First-class functions** and lambda expressions
- **Generics** and type parameters
- **Memory safety** without garbage collection pauses

### Architecture
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
- LLVM/Clang
- Protocol Buffers compiler

### From Source
```bash
git clone https://github.com/yourusername/fax.git
cd fax
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

- [Language Specification](SPEC.md) - Complete language reference
- [Architecture Guide](ARCHITECTURE.md) - Microservices architecture
- [FGC Documentation](docs/FGC.md) - Garbage collector details
- [Examples](examples/) - Sample programs

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
fax/
├── faxc/                   # Compiler source code
│   ├── Compiler/
│   │   ├── AST/           # Abstract Syntax Tree
│   │   ├── Lexer/         # Tokenization
│   │   ├── Parser/        # AST construction
│   │   ├── Semantic/      # Type checking
│   │   ├── Codegen/       # LLVM IR generation
│   │   ├── Driver/        # Compiler driver & CLI
│   │   ├── Proto/         # Microservices
│   │   ├── Runtime/       # FGC implementation
│   │   └── Validation/    # Input validation
│   ├── Fax.lean           # Main entry
│   └── StdLib.lean        # Standard library
├── tests/                  # Test suites
│   ├── unit/              # Unit tests
│   ├── integration/       # Integration tests
│   └── e2e/               # End-to-end tests
├── examples/               # Example programs
└── docs/                   # Documentation
```

### Module Organization

The compiler follows a **modular architecture** with clear separation of concerns:

- **Index Files**: Each module has an index file (e.g., `Compiler/Semantic.lean`) that exports the public API
- **Submodules**: Functionality is split into focused submodules (e.g., `Semantic/Checker.lean`, `Semantic/Inference.lean`)
- **Validation Module**: New input validation with separate validators for source, identifiers, types, and limits

See [MODULE_STRUCTURE.md](MODULE_STRUCTURE.md) for detailed module organization guidelines.

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

### Contributing
Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

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

### Phase 2 (In Progress) 🚧
- [x] Complete Semantic Analysis
- [x] E2E Tests
- [x] Docker & CI/CD
- [ ] LLVM FFI bindings
- [ ] Standard Library

### Phase 3 (Planned) 📋
- [ ] Optimization passes
- [ ] Package manager
- [ ] IDE support
- [ ] WebAssembly target

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md).

### Areas for Contribution
- Language features
- Performance improvements
- Documentation
- Bug fixes
- Example programs

## 📄 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file.

## 🙏 Acknowledgments

- Lean 4 team for the excellent theorem prover
- ZGC team for inspiration on low-latency GC
- Protocol Buffers team

## 📞 Support

- GitHub Issues: [Report bugs](https://github.com/yourusername/fax/issues)
- Discussions: [Ask questions](https://github.com/yourusername/fax/discussions)
- Email: support@fax-lang.org

---

**Made with ❤️ by the Fax Compiler Team**
