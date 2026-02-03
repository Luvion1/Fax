# Pipeline & Hub Architecture

The Fax compiler uses a **Polyglot Pipeline** architecture. Instead of a single monolithic process, it consists of several independent executables coordinated by a central Hub.

## The Hub (TypeScript)

Located at `faxc/src/hub/pipeline.ts`, the Hub is responsible for:
- Orchestrating the flow of data between components.
- Managing temporary files for inter-component communication.
- Timing each phase and reporting errors.
- Invoking the final linker (Zig CC).

## Communication Protocol

Components communicate using **JSON over Files/Stdout**. 

1. **Input:** The Hub writes the current state (Tokens or AST) to a temporary JSON file.
2. **Execution:** The Hub calls the component executable with the path to the JSON file as an argument.
3. **Output:** The component processes the input and prints the resulting JSON to `stdout`.
4. **Ingestion:** The Hub captures `stdout`, parses the JSON, and passes it to the next stage.

## Compilation Flow

1. **Source (`.fax`)** -> `Lexer` (Rust) -> **Tokens (JSON)**
2. **Tokens** -> `Parser` (Zig) -> **Untyped AST (JSON)**
3. **Untyped AST** -> `Sema` (Haskell) -> **Typed AST (JSON)**
4. **Typed AST** -> `Optimizer` (Python) -> **Optimized AST (JSON)**
5. **Optimized AST** -> `Codegen` (C++) -> **LLVM IR (`.ll`)**
6. **LLVM IR + Fgc Object** -> `Zig CC` -> **Native Binary**

## Error Handling

Each component is expected to report errors in a structured JSON format or via `stderr`. If a component returns a non-zero exit code or a JSON containing an `"error"` field, the Hub halts the pipeline and displays the error message.
