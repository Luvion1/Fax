# Fax Programming Language

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.8.0--modular-purple.svg)]()

Fax is a high-performance, polyglot-driven programming language. It leverages a unique modular pipeline where each stage is implemented in the most suitable language (Rust, Zig, Haskell, C++, and Node.js).

## Core Philosophy

- **Polyglot Design:** Use the best tool for the job.
- **Precise GC:** Custom generational garbage collector for predictable performance.
- **Simplicity:** Clean syntax inspired by modern systems languages.
- **Transparency:** Direct access to runtime internals like GC control.

## Quick Start

```bash
# Setup toolchain
ln -sf $(pwd)/faxt/main.py /usr/local/bin/faxt
chmod +x faxt/main.py

# Create project
faxt new hello_world
cd hello_world
faxt run
```

## Documentation

- [Language Guide](docs/language.md) - Syntax, types, and semantics.
- [Toolchain Manual](docs/tooling.md) - Mastering the `faxt` CLI.
- [Internal Architecture](docs/architecture.md) - Deep dive into the polyglot pipeline.
- [Memory Management](docs/memory.md) - How the Fax GC (FGC) works.

## License

Fax is released under the MIT License.
