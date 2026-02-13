# Fax-lang (GEMINI Context)

This `GEMINI.md` file provides essential context for interacting with the Fax-lang project, an experimental polyglot compiler system.

## Project Overview

**Fax-lang** is a high-performance, modular language system where each compilation stage is implemented in a different systems programming language to leverage their specific strengths:

*   **Lexer:** Rust (High-speed tokenization and UTF-8 handling)
*   **Parser:** Zig (Memory-efficient recursive descent parsing)
*   **Semantic Analysis (Sema):** Haskell (Rigorous type checking and validation)
*   **Optimizer:** Rust (Graph-based AST transformations)
*   **Codegen:** C++ (LLVM IR generation)
*   **Runtime (Fgc):** Zig (Custom ZGC-inspired Mark-Relocate Garbage Collector)
*   **Orchestration (Hub):** TypeScript/Node.js (Pipeline management)

The stages communicate via structured JSON data, allowing for a highly decoupled and modular architecture.

## Directory Structure

*   **`faxc/packages/`**: Contains the source code for all compiler components.
    *   **`lexer/`**: Rust implementation.
    *   **`parser/`**: Zig implementation.
    *   **`sema/`**: Haskell implementation.
    *   **`optimizer/`**: Rust implementation.
    *   **`codegen/`**: C++ implementation.
    *   **`runtime/`**: Zig implementation (Fgc).
    *   **`hub/`**: TypeScript orchestration logic.
*   **`std/`**: The Fax standard library (written in `.fax`).
*   **`tests/`**: Comprehensive test suites for the language.
*   **`scripts/`**: Utility scripts for building, testing, and linting.
*   **`Fax.toml`**: Project-level configuration.

## Building and Running

### Prerequisites
The project requires several toolchains:
*   Rust (Stable)
*   Zig (v0.13.0+)
*   GHC (Haskell)
*   CMake & C++ Compiler (GCC/Clang)
*   Node.js (v20+)
*   LLVM (v14+)

### Setup and Build
1.  **Install Dependencies:**
    ```bash
    ./install_dependencies.sh
    ```
2.  **Build the Entire Compiler:**
    ```bash
    ./build_compiler.sh
    ```
    This script builds all components (Lexer, Parser, Sema, Optimizer, Codegen, and Runtime) and places binaries in their respective `target/`, `zig-out/`, or `bin/` directories.

### Running the Compiler
The main entry point for compiling and running Fax source files is the `run_pipeline.sh` script:

```bash
./run_pipeline.sh <path/to/file.fax>
```

This script orchestrates the following flow:
1. `lexer` -> `temp_tokens.json`
2. `parser` -> `temp_ast.json`
3. `sema` -> `temp_typed.json`
4. `optimizer` -> `temp_optimized.json`
5. `codegen` -> `output.ll` (LLVM IR)
6. `zig cc` (Linking with Runtime) -> `output` (Native Executable)

## Development Conventions

*   **Communication:** Components exchange data using JSON. Ensure any changes to the data structures are reflected across all relevant components.
*   **Language-Specific Idioms:**
    *   **Rust:** Use `cargo build --release` and follow standard Clippy lints.
    *   **Zig:** Use `zig build` and `zig fmt`.
    *   **Haskell:** Use `ghc` for building the Sema component.
    *   **C++:** Use CMake for the Codegen build system.
*   **Documentation:** Technical documentation and code comments should be in **English**.
*   **User Preference:** The user prefers to communicate in **Indonesian (Bahasa Indonesia)**, but wants technical content and documentation to remain in **English**.

## Key Files
*   `README.md`: General project overview.
*   `DEVELOPMENT_GUIDE.md`: Detailed setup and development instructions.
*   `run_pipeline.sh`: The primary shell script driving the compilation process.
*   `build_compiler.sh`: The master build script for all components.
*   `faxc/packages/hub/src/orchestrator/pipeline.ts`: The modular TS orchestration logic.
