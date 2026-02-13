# 📠 Fax Programming Language Documentation

Welcome to the official documentation for **Fax**, a high-performance polyglot programming language compiler.

## 🚀 Quick Start

### Installation

```bash
git clone https://github.com/Luvion1/Fax.git
cd Fax
npm install
```

### Your First Program

Create `hello.fax`:

```fax
fn main() {
    let message = "Hello, Fax!";
    print(message);
}
```

Run it:

```bash
python3 faxt/main.py run hello.fax
```

## 📚 Documentation

| Guide | Description |
|-------|-------------|
| [Getting Started](getting-started.md) | Quick start guide |
| [Language Guide](language.md) | Syntax and features |
| [Architecture](architecture.md) | Compiler internals |
| [Memory Management](memory.md) | FGC garbage collector |
| [Tooling](tooling.md) | CLI reference |
| [Specification](specification.md) | Language specification |

## 🏗 Architecture

Fax uses a **polyglot compiler pipeline** where each stage is implemented in the most suitable language:

```
Source → Lexer → Parser → Sema → Optimizer → Codegen → Runtime
         (Rust)   (Zig)   (Haskell)  (Rust)     (C++)    (Zig)
```

## ✨ Features

- **Static Typing** with type inference
- **Generational Garbage Collector** (FGC)
- **Pattern Matching** with exhaustiveness checking
- **Control Flow**: if/elif/else, while, for loops
- **Structs** and data structures
- **10,000+ Test Cases** for reliability

## 🔧 Development

See [DEVELOPMENT.md](../DEVELOPMENT.md) for setup instructions.

## 📄 License

MIT License - See [LICENSE](../LICENSE)

---

*Built with ❤️ by the Fax team*
