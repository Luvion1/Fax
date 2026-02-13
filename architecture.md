# Compiler Architecture

The Fax compiler is a modular pipeline orchestrated by a TypeScript Hub.

## Pipeline Stages

1.  **Lexer (Rust):** Tokenizes source text.
2.  **Parser (Zig):** Builds an untyped AST.
3.  **Sema (Haskell):** Type inference and semantic validation.
4.  **Optimizer (Python):** Constant folding and IR cleanup.
5.  **Codegen (C++):** Low-level LLVM IR generation.
6.  **Runtime (Zig):** Memory management and core primitives.
7.  **Linker (Zig CC):** Final binary assembly.

## Inter-Process Communication

Stages communicate using **JSON over standard I/O**. This allows each component to be swapped or tested in isolation using `faxt run --stage`.

```bash
# Example of inspecting intermediate AST
faxt run src/main.fax --check > ast.json
faxt inspect ast.json
```
