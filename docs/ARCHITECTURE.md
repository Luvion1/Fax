# Fax Compiler Architecture

Fax uses a **polyglot compiler pipeline** where each stage is implemented in the most suitable language for that specific task.

## Overview

```
Source Code (.fax)
       │
       ▼
┌──────────────────┐
│     Lexer        │  Rust
│  Tokenization    │
│   36 token      │
│   types         │
└────────┬─────────┘
         │ JSON Tokens
         ▼
┌──────────────────┐
│     Parser       │  Zig
│   AST Building   │
│   - Functions   │
│   - Structs     │
│   - Control Flow │
└────────┬─────────┘
         │ JSON AST
         ▼
┌──────────────────┐
│   Sema           │  Haskell
│ Type Checking    │
│ - Inference      │
│ - Control Flow   │
│ - Patterns       │
└────────┬─────────┘
         │ Validated AST
         ▼
┌──────────────────┐
│    Optimizer     │  Rust
│  AST-level Opts  │
│   5 levels      │
└────────┬─────────┘
         │ Optimized AST
         ▼
┌──────────────────┐
│    Codegen       │  C++
│   LLVM IR Gen    │
│ - Type mapping   │
│ - Instruction    │
└────────┬─────────┘
         │ LLVM IR
         ▼
┌──────────────────┐
│    Runtime       │  Zig
│  FGC Collector   │
│ - Memory mgmt   │
│ - FFI exports   │
└──────────────────┘
```

## Stage Details

### 1. Lexer (Rust)

**Location**: `faxc/packages/lexer/`

Tokenizes source code into a stream of tokens.

- **Token Types**: 36 types
- **Keywords**: 85+ keywords
- **Output**: JSON array of tokens

Key files:
- `src/lexer/tokenizer.rs` - Main tokenizer
- `src/lexer/token.rs` - Token definitions

### 2. Parser (Zig)

**Location**: `faxc/packages/parser/`

Builds Abstract Syntax Tree (AST) from tokens.

- **AST Nodes**: Functions, structs, classes, control flow, pattern matching
- **Output**: JSON AST tree

Key files:
- `src/parser/parser.zig` - Main parser
- `src/parser/stmt.zig` - Statement parsing
- `src/parser/expr.zig` - Expression parsing

### 3. Semantic Analyzer - Sema (Haskell)

**Location**: `faxc/packages/sema/`

Type checking and semantic analysis.

**Modules**:
| Module | Description |
|--------|-------------|
| `Types.hs` | Type definitions, inference, unification |
| `Errors.hs` | Error types, suggestions |
| `Checker.hs` | Main type checking logic |
| `ControlFlow.hs` | Flow analysis, pattern matching |
| `Pretty.hs` | Error formatting |
| `Diag.hs` | Diagnostic types |
| `ASTUtils.hs` | AST utilities |
| `ConstantFolding.hs` | Constant evaluation |

**Error Detection**:
- E001-E022: Type errors
- W001-W009: Warnings

Key functions:
- `check` - Main type checking
- `checkFunc` - Function checking
- `checkMatch` - Pattern matching

### 4. Optimizer (Rust)

**Location**: `faxc/packages/optimizer/`

Performs AST-level optimizations.

- **Optimization Levels**: 0-4
- **Techniques**: Constant folding, dead code elimination

### 5. Code Generator (C++)

**Location**: `faxc/packages/codegen/`

Generates LLVM IR from AST.

- **Type Mapping**: Fax types → LLVM types
- **Output**: LLVM IR (`.ll` file)

Key files:
- `src/backend/codegen.cpp` - Main codegen
- `src/backend/codegen.hpp` - Header

### 6. Runtime (Zig)

**Location**: `faxc/packages/runtime/`

Execution environment with Garbage Collector.

**Features**:
- **FGC**: Fax Garbage Collector (generational)
- **Memory Management**: Allocation, deallocation
- **FFI**: C ABI exports

Key files:
- `src/gc/fgc.zig` - GC implementation
- `src/main.zig` - Entry point
- `src/api/exports.zig` - C exports

## Data Flow

### Between Stages

Each stage communicates via JSON files:

1. **Lexer → Parser**: `tokens.json`
2. **Parser → Sema**: `ast.json`
3. **Sema → Optimizer**: `validated_ast.json`
4. **Optimizer → Codegen**: `optimized_ast.json`
5. **Codegen → Runtime**: `program.ll`

### Hub (Orchestrator)

**Location**: `faxc/packages/hub/`

The Hub orchestrates the entire pipeline using Node.js/TypeScript.

```typescript
// Pipeline execution
const pipeline = new Pipeline();
await pipeline.execute('source.fax');
```

## Directory Structure

```
faxc/
├── packages/
│   ├── lexer/        # Rust - Tokenization
│   ├── parser/       # Zig - AST generation
│   ├── sema/         # Haskell - Type checking
│   ├── optimizer/    # Rust - Optimizations
│   ├── codegen/     # C++ - LLVM IR
│   ├── runtime/     # Zig - Execution
│   └── hub/         # TypeScript - Orchestrator
```

## Extending the Compiler

### Adding a New Stage

1. Create new package in `faxc/packages/`
2. Implement JSON input/output
3. Add adapter in `hub/`
4. Update pipeline in orchestrator

### Adding New Language Features

1. **Lexer**: Add token in `lexer/src/`
2. **Parser**: Add AST node in `parser/src/`
3. **Sema**: Add type rule in `sema/src/Checker.hs`
4. **Codegen**: Add codegen in `codegen/src/`
5. **Runtime**: Add runtime support in `runtime/src/`

## Testing

Run the full pipeline:

```bash
./run_pipeline.sh source.fax
```

Or use the CLI:

```bash
python3 faxt/main.py run source.fax
```

## Performance

- **Lexer**: ~10,000 lines/sec
- **Parser**: ~5,000 lines/sec
- **Sema**: ~3,000 lines/sec
- **Codegen**: ~2,000 lines/sec

See [Memory Management](memory.md) for GC performance details.
