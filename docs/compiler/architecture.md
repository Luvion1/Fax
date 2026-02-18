# Fax Compiler Architecture

Overview of the Fax compiler's internal architecture and design.

## Table of Contents

1. [Overview](#overview)
2. [Compiler Pipeline](#compiler-pipeline)
3. [Component Details](#component-details)
4. [Data Flow](#data-flow)
5. [Key Design Decisions](#key-design-decisions)

---

## Overview

The Fax compiler (`faxc`) is a ahead-of-time (AOT) compiler that translates Fax source code into native machine code via LLVM IR.

```
┌─────────────────────────────────────────────────────────────────┐
│                        Fax Compiler Pipeline                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Source    ┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐   ┌─────────┐ │
│  (.fax) ──►│ Lex │──►│ Par │──►│ Sem │──►│ MIR │──►│  Code   │ │
│            └─────┘   └─────┘   └─────┘   └─────┘   │  Gen    │ │
│                                                    └────┬────┘ │
│                                                         │       │
│  Binary    ┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐       ▼       │
│  (.out) ◄──│ Link│◄───│ Opt │◄───│ LIR │◄───│ LLVM IR│◄────────┘ │
│            └─────┘   └─────┘   └─────┘   └─────┘                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Compiler Pipeline

### Stage 1: Lexical Analysis (Lexer)

**Crate:** `faxc-lex`

The lexer converts source code into a stream of tokens.

```
Source Code          Tokens
──────────           ──────
fn main()    ──►    [FN] [IDENT:main] [LPAREN] [RPAREN] [LBRACE] [RBRACE]
```

**Responsibilities:**
- Tokenize input characters
- Handle whitespace and comments
- Report lexical errors
- Support Unicode identifiers

### Stage 2: Parsing

**Crate:** `faxc-par`

The parser converts tokens into an Abstract Syntax Tree (AST).

```
Tokens              AST
──────              ───
[FN] [IDENT:main]   FunctionDecl
[LPAREN] [RPAREN]     name: "main"
[LBRACE] [RBRACE]     params: []
                      body: Block { ... }
```

**Responsibilities:**
- Build AST from token stream
- Enforce grammar rules
- Report syntax errors with helpful messages
- Handle operator precedence

### Stage 3: Semantic Analysis

**Crate:** `faxc-sem`

The semantic analyzer validates the AST and builds symbol tables.

```
AST                 Annotated AST
───                 ─────────────
FunctionDecl        FunctionDecl
  name: "main"  ──►   name: "main"
  body: ...           return_type: Unit
                      symbols: { ... }
                      type_checked: true
```

**Responsibilities:**
- Type checking and inference
- Name resolution
- Symbol table management
- Semantic error reporting

### Stage 4: Mid-level IR (MIR)

**Crate:** `faxc-mir`

MIR is a simplified, lower-level representation optimized for analysis.

```
Annotated AST       MIR
───────────────     ───
FunctionDecl        Function
  ...                 basic_blocks: [ ... ]
                      locals: [ ... ]
                      terminators: [ ... ]
```

**Responsibilities:**
- Convert AST to SSA form
- Build control flow graph
- Perform mid-level optimizations
- Prepare for code generation

### Stage 5: Code Generation

**Crate:** `faxc-gen`

Generates LLVM IR from MIR.

```
MIR                 LLVM IR
───                 ───────
Function            define i32 @main() {
  ...                 entry:
                        ret i32 0
                      }
```

**Responsibilities:**
- Translate MIR to LLVM IR
- Handle type conversions
- Generate calling conventions
- Emit debug information

### Stage 6: Low-level IR (LIR)

**Crate:** `faxc-lir`

LIR handles platform-specific lowering.

**Responsibilities:**
- Target-specific optimizations
- ABI compliance
- Register allocation hints
- Instruction selection

### Stage 7: LLVM Backend

The LLVM backend handles:
- Optimization passes
- Register allocation
- Instruction scheduling
- Object code generation

---

## Component Details

### Workspace Structure

```
faxc/
├── Cargo.toml              # Workspace configuration
├── crates/
│   ├── faxc-util/          # Shared utilities
│   ├── faxc-lex/           # Lexer
│   ├── faxc-par/           # Parser
│   ├── faxc-sem/           # Semantic analysis
│   ├── faxc-mir/           # Mid-level IR
│   ├── faxc-lir/           # Low-level IR
│   ├── faxc-gen/           # Code generation
│   ├── faxc-drv/           # Compiler driver
│   └── fgc/                # Garbage collector
├── examples/               # Example Fax programs
├── scripts/                # Build and test scripts
└── tests/                  # Integration tests
```

### Garbage Collector (FGC)

The Fax Garbage Collector (FGC) is a concurrent, generational collector:

```
┌─────────────────────────────────────────┐
│           FGC Architecture               │
├─────────────────────────────────────────┤
│                                          │
│  ┌─────────────┐    ┌─────────────────┐ │
│  │  Allocator  │───►│  Heap Manager   │ │
│  │  (TLAB)     │    │                 │ │
│  └─────────────┘    └────────┬────────┘ │
│                              │          │
│  ┌─────────────┐    ┌────────▼────────┐ │
│  │  Finalizer  │◄───│  GC Core        │ │
│  │             │    │  - Mark         │ │
│  └─────────────┘    │  - Sweep        │ │
│                     │  - Compact      │ │
│                     └─────────────────┘ │
│                                          │
└─────────────────────────────────────────┘
```

---

## Data Flow

### Compilation Flow

```
User Input
    │
    ▼
┌─────────────────┐
│  Compiler Driver │
│  (faxc-drv)      │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌───────┐ ┌──────────┐
│ Parse │ │  Check   │
│ Source│ │  Options │
└───┬───┘ └──────────┘
    │
    ▼
┌───────────┐
│   Lexer   │
│ (faxc-lex)│
└─────┬─────┘
      │
      ▼
┌───────────┐
│  Parser   │
│ (faxc-par)│
└─────┬─────┘
      │
      ▼
┌───────────┐
│ Semantic  │
│ (faxc-sem)│
└─────┬─────┘
      │
      ▼
┌───────────┐
│    MIR    │
│ (faxc-mir)│
└─────┬─────┘
      │
      ▼
┌───────────┐
│   Code    │
│   Gen     │
│ (faxc-gen)│
└─────┬─────┘
      │
      ▼
┌───────────┐
│   LLVM    │
│  Backend  │
└─────┬─────┘
      │
      ▼
┌───────────┐
│  Output   │
│  Binary   │
└───────────┘
```

---

## Key Design Decisions

### 1. Multi-Crate Architecture

**Decision:** Split compiler into multiple crates

**Rationale:**
- Clear separation of concerns
- Independent testing of components
- Better compile times for development
- Reusability of components

### 2. LLVM as Backend

**Decision:** Use LLVM for code generation

**Rationale:**
- Mature optimization passes
- Cross-platform support
- Active community and documentation
- High-quality code generation

### 3. Garbage Collection

**Decision:** Include built-in garbage collector

**Rationale:**
- Simplifies memory management for users
- Enables functional programming patterns
- Reduces common bugs (use-after-free, etc.)
- Modern language expectation

### 4. Functional-First Design

**Decision:** Prioritize functional programming features

**Rationale:**
- Expressive and concise code
- Easier reasoning about programs
- Better composability
- Modern language trends

---

## Performance Considerations

### Compilation Speed

- Incremental compilation support
- Parallel crate compilation
- Efficient data structures

### Runtime Performance

- LLVM optimizations
- Zero-cost abstractions where possible
- Efficient GC algorithm
- TLAB allocation

---

## Testing Strategy

```
┌─────────────────────────────────────────┐
│          Testing Pyramid                 │
├─────────────────────────────────────────┤
│                  /\                      │
│                 /  \                     │
│                / E2E \                   │
│               /────────\                 │
│              /          \                │
│             / Integration \              │
│            /──────────────\              │
│           /                \             │
│          /    Unit Tests    \            │
│         /────────────────────\           │
│        /                      \          │
│       ──────────────────────────         │
│                                          │
└─────────────────────────────────────────┘
```

### Unit Tests
- Individual function testing
- Crate-level test suites
- Property-based testing

### Integration Tests
- End-to-end compilation
- Cross-crate interactions
- Regression tests

### E2E Tests
- Full program compilation
- Output verification
- Performance benchmarks

---

## Future Directions

### Short-term
- [ ] Complete generics implementation
- [ ] Add trait system
- [ ] Improve error messages
- [ ] Add more optimizations

### Long-term
- [ ] Async/await support
- [ ] Better IDE integration (LSP)
- [ ] Package manager
- [ ] Standard library expansion

---

<div align="center">

**Understanding the Fax compiler internals** 🔧

</div>
