# 📠 Fax-lang

> **The Polyglot Compiler Experiment.**  
> A high-performance, modular language system where every compilation stage is a showcase of modern systems programming.

[![CI](https://github.com/Luvion1/Fax/actions/workflows/ci.yml/badge.svg)](https://github.com/Luvion1/Fax/actions/workflows/ci.yml)
[![Version](https://img.shields.io/badge/version-v0.0.1--alpha-blue)](https://github.com/Luvion1/Fax/releases/tag/v0.0.1-alpha)
[![Status](https://img.shields.io/badge/Status-Active-brightgreen.svg)]()
[![Runtime](https://img.shields.io/badge/Runtime-Fgc-blue.svg)]()
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

---

## 🚀 Overview

Fax is not just a language; it's a multi-language orchestration. By leveraging the strengths of **Rust, Zig, Haskell, C++, and Python**, Fax achieves a unique balance between safety, mathematical correctness, and raw performance.

### Key Pillars
- **🏗️ Modular by Design**: Every stage (Lexer to Codegen) is an independent micro-service communicating via structured JSON.
- **💎 Fgc Runtime**: A state-of-the-art ZGC-inspired Garbage Collector written in Zig.
- **🛡️ Type Safe**: Semantic analysis powered by Haskell's rigorous type system.
- **⚡ LLVM Powered**: Generates highly optimized native machine code.

---

## 🛠️ Architecture at a Glance

```mermaid
graph TD
    A[Source .fax] -->|Rust| B(Lexer)
    B -->|JSON Tokens| C(Parser)
    C -->|Zig| D{AST}
    D -->|Haskell| E(Sema)
    E -->|Python| F(Optimizer)
    F -->|C++| G(Codegen)
    G -->|LLVM IR| H(Zig CC / Linker)
    H -->|Native| I[Executable]
```

| Component | Language | Role |
| :--- | :--- | :--- |
| **Lexer** | 🦀 Rust | High-speed tokenization & UTF-8 handling. |
| **Parser** | ⚡ Zig | Memory-efficient Recursive Descent parsing. |
| **Sema** | λ Haskell | Recursive type checking & semantic validation. |
| **Optimizer**| 🐍 Python | Graph-based AST transformations. |
| **Codegen** | ⚙️ C++ | LLVM IR generation & Stack Map emission. |
| **Runtime** | ⚡ Zig | **Fgc**: Colored, Mark-Relocate Garbage Collector. |

---

## 📦 Getting Started

### 1. Prerequisites
Ensure you have the following toolchains installed:
- **Rust** (Cargo)
- **Zig** (0.15.2+)
- **Haskell** (GHC)
- **C++** (GCC/Clang)
- **Node.js** (for the Hub)

### 2. Installation
```bash
git clone https://github.com/fax-lang/fax
cd fax/faxc
npm install
```

### 3. Your First Program
Create `hello.fax`:
```fax
fn main() {
    print("Hello, the future of polyglot coding!")
}
```
Run it:
```bash
npx ts-node src/hub/pipeline.ts hello.fax
```

---

## 🗺️ Roadmap
- [x] Recursive Descent Parser in Zig.
- [x] Fgc (ZGC-style) Mark-Relocate runtime.
- [ ] **Next**: Trait system and Generic types.
- [ ] **Next**: Concurrency primitives (Goroutine-style).
- [ ] **Future**: Self-hosting (Fax written in Fax).

---

## 📄 Documentation
- [User Guide](docs/user_guide.md) - Learn how to code in Fax.
- [Internals Deep Dive](docs/internals/overview.md) - How the compiler works.
- [Fgc Architecture](docs/fgc_architecture.md) - Understanding the memory management.

---
*Built with ❤️ by the Fax-lang Team.*
