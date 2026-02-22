# Fax Compiler Documentation

![Fax Arctic Fox](../faxc/docs/rubah-arktik.svg =120x120)

Welcome to the official documentation for the Fax programming language and compiler.

<!-- Source: faxc/Cargo.toml, README.md -->

## 📚 Documentation Overview

| Section | Description |
|---------|-------------|
| [Getting Started](docs/getting-started/) | Installation, quick tour, and your first Fax program |
| [Language Guide](docs/language-guide/) | Comprehensive language features and syntax |
| [Compiler Documentation](docs/compiler/) | Compiler architecture, building, and contributing |
| [RFCs](docs/rfcs/) | Request for Comments and design proposals |

---

## 🚀 Quick Links

### For New Users
- [Installation Guide](docs/getting-started/installation.md) - Get Fax up and running (includes LLVM 20 setup)
- [Quick Tour](docs/getting-started/quick-tour.md) - Learn the basics in 10 minutes
- [Hello World](docs/getting-started/hello-world.md) - Your first Fax program

### For Developers
- [Language Specification](../../SPEC.md) - Complete grammar and semantics
- [Architecture Overview](docs/compiler/architecture.md) - How the compiler works
- [Building Guide](docs/compiler/building.md) - Build from source with LLVM 20
- [Contributing Guide](../../CONTRIBUTING.md) - How to contribute to Fax

### For Contributors
- [Building from Source](docs/compiler/building.md) - Build the compiler locally
- [Contributing to Compiler](docs/compiler/contributing.md) - Compiler development guide
- [Code Style](../../faxc/rustfmt.toml) - Rust formatting configuration

---

## 📖 Documentation Sections

### Getting Started

```
docs/getting-started/
├── installation.md      # Installing Fax and LLVM 20
├── quick-tour.md        # Language overview and features
└── hello-world.md       # Your first Fax program
```

### Language Guide

```
docs/language-guide/
├── basics.md            # Variables, types, and expressions
├── types.md             # Type system deep dive
├── functions.md         # Functions and lambdas
├── control-flow.md      # Conditionals and loops
├── pattern-matching.md  # Match expressions
├── data-types.md        # Structs, enums, and tuples
├── modules.md           # Module system and visibility
└── advanced.md          # Advanced language features
```

### Compiler Documentation

```
docs/compiler/
├── architecture.md      # Compiler pipeline overview
├── building.md          # Building from source (LLVM 20)
├── contributing.md      # Contributing to the compiler
├── testing.md           # Testing infrastructure
└── debugging.md         # Debugging tips and tools
```

### RFCs (Request for Comments)

```
docs/rfcs/
├── template.md          # RFC template
├── 0001-template.md     # Example RFC
└── README.md            # RFC process documentation
```

---

## 📋 Additional Resources

### External Documentation
- [Rust Programming Language](https://doc.rust-lang.org/book/) - For compiler contributors
- [LLVM 20 Documentation](https://llvm.org/docs/) - For code generation
- [Cargo Book](https://doc.rust-lang.org/cargo/) - For Rust package management

### Community
- [GitHub Discussions](https://github.com/Luvion1/Fax/discussions) - Ask questions
- [GitHub Issues](https://github.com/Luvion1/Fax/issues) - Report bugs
- [Security Policy](../../SECURITY.md) - Report vulnerabilities

---

## 🔍 Search Documentation

Use GitHub's search to find specific topics:
- `repo:Luvion1/Fax path:docs/ <search term>`
- `repo:Luvion1/Fax path:*.md <search term>`

---

## 📝 Contributing to Documentation

We welcome documentation contributions! See our [Contributing Guide](../../CONTRIBUTING.md) for details.

### Documentation Guidelines
- Use clear, concise language
- Include code examples where helpful
- Keep examples up to date with the latest version
- Use proper Markdown formatting
- Add internal links where relevant
- Keep LLVM version references consistent (LLVM 20)

---

## 📄 License

Documentation is licensed under the same terms as the Fax Compiler:
- **MIT License** or **Apache License 2.0** (at your option)

See [LICENSE](../../LICENSE) for details.

---

<div align="center">

**Fax Compiler Documentation** v0.0.2 pre-alpha

Made with ❤️ by the Fax Team

</div>
