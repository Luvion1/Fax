# Fax Compiler Architecture

## Document Information

- **Version**: 1.0.0
- **Status**: Draft
- **Last Updated**: 2026-02-17
- **Authors**: Fax Team

---

## Table of Contents

1. [Overview](#overview)
2. [System Architecture](#system-architecture)
3. [Module Structure](#module-structure)
4. [Compiler Pipeline](#compiler-pipeline)
5. [Dependency Graph](#dependency-graph)
6. [Development Strategy](#development-strategy)
7. [Technical Decisions](#technical-decisions)
8. [Quality Standards](#quality-standards)

---

## Overview

### What is Fax?

Fax is a modern, functional-first systems programming language that combines:
- **Simple syntax** inspired by Go
- **Powerful type system** inspired by Rust
- **Garbage-collected memory management** via FGC
- **Native performance** via LLVM compilation

### Design Goals

| Goal | Description |
|------|-------------|
| **Simple & Clean** | Minimal syntax, maximum readability |
| **Functional-first** | First-class functions, immutability by default |
| **Modern** | Type inference, pattern matching, ADTs |
| **Fast** | Compiles to native code via LLVM |
| **Safe** | Garbage collection, no manual memory management |

### Key Features

- Static typing with type inference
- First-class functions and lambdas
- Algebraic Data Types (ADTs) via enums
- Pattern matching
- Async/await support
- Garbage-collected memory (FGC)
- Zero-cost abstractions where possible

---

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FAX COMPILER SYSTEM                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         COMPILER (faxc)                              │    │
│  │                                                                       │    │
│  │  Source (.fax) → [Frontend] → [Middle-end] → [Backend] → Executable  │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         RUNTIME (fgc)                                │    │
│  │                                                                       │    │
│  │  Garbage Collector: Concurrent, low-latency memory management        │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    STANDARD LIBRARY (std)                            │    │
│  │                                                                       │    │
│  │  Core, Alloc, Collections, IO, Async                                 │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Three-Layer Architecture

The Fax compiler follows a **three-layer architecture** with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────────┐
│                     FRONTEND (Source → HIR)                     │
│  ┌───────────┐    ┌───────────┐    ┌───────────┐               │
│  │   Lexer   │ →  │  Parser   │ →  │ Semantic  │               │
│  │  (tokens) │    │   (AST)   │    │  (HIR)    │               │
│  └───────────┘    └───────────┘    └───────────┘               │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   MIDDLE-END (HIR → LIR)                        │
│  ┌───────────┐    ┌───────────┐    ┌───────────┐               │
│  │    MIR    │ →  │   Opt     │ →  │    LIR    │               │
│  │   (SSA)   │    │ (passes)  │    │ (regs)    │               │
│  └───────────┘    └───────────┘    └───────────┘               │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    BACKEND (LIR → Binary)                       │
│  ┌───────────┐    ┌───────────┐    ┌───────────┐               │
│  │  Codegen  │ →  │  Linker   │ →  │ Executable│               │
│  │  (LLVM)   │    │           │    │           │               │
│  └───────────┘    └───────────┘    └───────────┘               │
└─────────────────────────────────────────────────────────────────┘
```

---

## Module Structure

### Folder Organization

```
faxc/
├── Cargo.toml                          # Workspace root
├── README.md
├── CONTRIBUTING.md
├── rust-toolchain.toml
│
├── crates/
│   ├── # ===============================
│   ├── # CORE UTILITIES (Foundation)
│   ├── # ===============================
│   │
│   ├── faxc-util/                      # [CORE] Shared utilities
│   │   └── src/
│   │       ├── lib.rs                  # Re-exports
│   │       ├── arena.rs                # Bump allocator, arena types
│   │       ├── symbol.rs               # String interning
│   │       ├── span.rs                 # Source locations, spans
│   │       ├── diagnostic.rs           # Error/warning reporting
│   │       ├── graph.rs                # Graph data structures
│   │       └── intern.rs               # General interning
│   │
│   ├── faxc-err/                       # [CORE] Error types & diagnostics
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs                # Compiler error types
│   │       ├── diagnostic.rs           # Diagnostic rendering
│   │       ├── codes.rs                # Error codes
│   │       └── handler.rs              # Error handler
│   │
│   ├── # ===============================
│   ├── # FRONTEND (Source → HIR)
│   ├── # ===============================
│   │
│   ├── faxc-ast/                       # [SHARED] Shared AST types
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ast.rs                  # Core AST types
│   │       ├── visit.rs                # AST visitor trait
│   │       └── fold.rs                 # AST folding trait
│   │
│   ├── faxc-lex/                       # [FRONTEND] Lexical analysis
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── lexer.rs                # Main lexer
│   │       ├── token.rs                # Token definitions
│   │       ├── cursor.rs               # Character cursor
│   │       └── unicode.rs              # Unicode handling
│   │
│   ├── faxc-par/                       # [FRONTEND] Parsing
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs               # Recursive descent parser
│   │       ├── ast.rs                  # AST construction
│   │       ├── expr.rs                 # Expression parsing (Pratt)
│   │       ├── stmt.rs                 # Statement parsing
│   │       ├── item.rs                 # Item parsing
│   │       └── recovery.rs             # Error recovery
│   │
│   ├── faxc-sem/                       # [FRONTEND] Semantic analysis
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── analyzer.rs             # Semantic analyzer
│   │       ├── hir.rs                  # High-level IR
│   │       ├── resolve.rs              # Name resolution
│   │       ├── infer.rs                # Type inference
│   │       ├── check.rs                # Type checking
│   │       ├── scope.rs                # Scope management
│   │       └── traits.rs               # Trait resolution
│   │
│   ├── # ===============================
│   ├── # MIDDLE-END (HIR → LIR)
│   ├── # ===============================
│   │
│   ├── faxc-mir/                       # [MIDDLE-END] Mid-level IR
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ir.rs                   # MIR definitions
│   │       ├── builder.rs              # MIR builder from HIR
│   │       ├── ssa.rs                  # SSA construction
│   │       ├── cfg.rs                  # Control flow graph
│   │       ├── transform/              # MIR transformations
│   │       │   ├── mod.rs
│   │       │   ├── async.rs            # Async/await lowering
│   │       │   └── borrow.rs           # Borrow checking
│   │       └── verify.rs               # MIR verification
│   │
│   ├── faxc-opt/                       # [MIDDLE-END] Optimization passes
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pass.rs                 # Pass manager
│   │       ├── inline.rs               # Inlining
│   │       ├── const_fold.rs           # Constant folding
│   │       ├── dce.rs                  # Dead code elimination
│   │       ├── gvn.rs                  # Global value numbering
│   │       ├── licm.rs                 # Loop invariant motion
│   │       └── simplify.rs             # CFG simplification
│   │
│   ├── faxc-lir/                       # [MIDDLE-END] Low-level IR
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ir.rs                   # LIR definitions
│   │       ├── lower.rs                # MIR→LIR lowering
│   │       ├── regalloc.rs             # Register allocation
│   │       ├── abi.rs                  # ABI handling
│   │       └── stack.rs                # Stack frame layout
│   │
│   ├── # ===============================
│   ├── # BACKEND (LIR → Binary)
│   ├── # ===============================
│   │
│   ├── faxc-codegen/                   # [BACKEND] Code generation
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── generator.rs            # Main code generator
│   │       ├── llvm/                   # LLVM backend (Inkwell)
│   │       │   ├── mod.rs
│   │       │   ├── module.rs           # LLVM module
│   │       │   ├── function.rs         # Function generation
│   │       │   ├── instr.rs            # Instruction selection
│   │       │   └── debug.rs            # Debug info (DWARF)
│   │       └── target/                 # Target-specific codegen
│   │           ├── mod.rs
│   │           ├── x86_64.rs
│   │           └── aarch64.rs
│   │
│   ├── faxc-link/                      # [BACKEND] Linking
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── linker.rs               # Linker wrapper
│   │       ├── resolve.rs              # Symbol resolution
│   │       └── archive.rs              # Static archive handling
│   │
│   ├── # ===============================
│   ├── # RUNTIME
│   ├── # ===============================
│   │
│   ├── fgc/                            # [RUNTIME] Garbage Collector
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── gc.rs                   # GC core
│   │       ├── heap.rs                 # Heap management
│   │       ├── mark.rs                 # Marking
│   │       ├── sweep.rs                # Sweeping
│   │       └── concurrent.rs           # Concurrent GC
│   │
│   ├── faxc-rt/                        # [RUNTIME] Runtime library
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── alloc.rs                # Allocation routines
│   │       ├── string.rs               # String runtime
│   │       ├── array.rs                # Array runtime
│   │       └── panic.rs                # Panic handling
│   │
│   ├── # ===============================
│   ├── # DRIVER & TOOLING
│   ├── # ===============================
│   │
│   └── faxc-drv/                       # [DRIVER] Compiler driver
│       └── src/
│           ├── main.rs                 # Binary entry point
│           ├── lib.rs
│           ├── config.rs               # Configuration
│           ├── session.rs              # Compilation session
│           ├── pipeline.rs             # Phase orchestration
│           ├── incremental.rs          # Incremental compilation
│           └── cli.rs                  # Command-line interface
│
├── std/                                # Standard Library
│   ├── lib.fax
│   ├── core/
│   ├── alloc/
│   ├── collections/
│   ├── io/
│   └── async/
│
├── tests/
│   ├── unit/                           # Unit tests per crate
│   ├── integration/                    # Cross-crate integration
│   ├── ui/                             # UI tests (error messages)
│   ├── runtime/                        # Runtime/GC tests
│   └── bench/                          # Benchmarks
│
├── tools/                              # Developer Tools (future)
│   ├── faxc-lsp/                       # Language Server
│   ├── faxc-fmt/                       # Code Formatter
│   └── faxc-doc/                       # Documentation Generator
│
└── docs/
    ├── arch/                           # Architecture documentation
    ├── spec/                           # Language specification
    ├── api/                            # Crate API docs
    └── dev/                            # Developer guide
```

---

## Compiler Pipeline

### Phase Overview

```
Source (.fax)
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 1: LEXICAL ANALYSIS (faxc-lex)                            │
│ Input:  Source code (String)                                    │
│ Output: Token stream (Vec<Token>)                               │
│ Tasks:  Tokenization, keyword recognition, literal parsing      │
└─────────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 2: PARSING (faxc-par)                                     │
│ Input:  Token stream                                            │
│ Output: Abstract Syntax Tree (AST)                              │
│ Tasks:  Syntax analysis, AST construction, error recovery       │
└─────────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 3: SEMANTIC ANALYSIS (faxc-sem)                           │
│ Input:  AST                                                     │
│ Output: High-level IR (HIR)                                     │
│ Tasks:  Name resolution, type checking, type inference          │
└─────────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 4: MIR GENERATION (faxc-mir)                              │
│ Input:  HIR                                                     │
│ Output: Mid-level IR (MIR) in SSA form                          │
│ Tasks:  CFG construction, SSA translation, async lowering       │
└─────────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 5: OPTIMIZATION (faxc-opt)                                │
│ Input:  MIR                                                     │
│ Output: Optimized MIR                                           │
│ Tasks:  Constant folding, DCE, inlining, LICM                   │
└─────────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 6: LIR GENERATION (faxc-lir)                              │
│ Input:  Optimized MIR                                           │
│ Output: Low-level IR (LIR) with virtual registers               │
│ Tasks:  PHI elimination, instruction selection                  │
└─────────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 7: REGISTER ALLOCATION (faxc-lir)                         │
│ Input:  LIR with virtual registers                              │
│ Output: LIR with physical registers                             │
│ Tasks:  Graph coloring, spill code generation                   │
└─────────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 8: CODE GENERATION (faxc-codegen)                         │
│ Input:  LIR with physical registers                             │
│ Output: LLVM IR → Object file                                   │
│ Tasks:  LLVM IR generation, optimization, emission              │
└─────────────────────────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 9: LINKING (faxc-link)                                    │
│ Input:  Object files + libraries                                │
│ Output: Executable binary                                       │
│ Tasks:  Symbol resolution, static/dynamic linking               │
└─────────────────────────────────────────────────────────────────┘
    │
    ▼
Executable
```

### IR Hierarchy

```
┌─────────────────────────────────────────────────────────────────┐
│                         IR LEVELS                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  HIR (High-level IR)                                            │
│  ├── Typed expressions                                          │
│  ├── Resolved names (DefId)                                     │
│  ├── High-level constructs (match, if, while)                   │
│  └── Close to source language                                   │
│                                                                  │
│  ↓ (lowering)                                                   │
│                                                                  │
│  MIR (Mid-level IR) - SSA Form                                  │
│  ├── Three-address code                                         │
│  ├── Explicit control flow (CFG)                                │
│  ├── PHI nodes for merge points                                 │
│  ├── Async/await as state machines                              │
│  └── Platform-independent                                       │
│                                                                  │
│  ↓ (lowering)                                                   │
│                                                                  │
│  LIR (Low-level IR)                                             │
│  ├── Machine-like instructions                                  │
│  ├── Virtual registers                                          │
│  ├── Calling convention applied                                 │
│  ├── Stack frame layout                                         │
│  └── Close to machine code                                      │
│                                                                  │
│  ↓ (register allocation)                                        │
│                                                                  │
│  LIR (Physical Registers)                                       │
│  ├── Physical registers (RAX, RBX, etc.)                        │
│  ├── Memory operands                                            │
│  └── Ready for code generation                                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Dependency Graph

### Crate Dependencies

```
                                    ┌─────────────┐
                                    │  faxc-util  │
                                    │  (CORE)     │
                                    │  - Symbol   │
                                    │  - Span     │
                                    │  - Diag     │
                                    └──────┬──────┘
                                           │
                                    ┌──────▼──────┐
                                    │  faxc-err   │
                                    │  (CORE)     │
                                    │  - Errors   │
                                    │  - Codes    │
                                    └──────┬──────┘
                                           │
                    ┌──────────────────────┼──────────────────────┐
                    │                      │                      │
             ┌──────▼──────┐       ┌──────▼──────┐       ┌──────▼──────┐
             │  faxc-ast   │       │   fgc       │       │  faxc-rt    │
             │  (SHARED)   │       │  (RUNTIME)  │       │  (RUNTIME)  │
             │  - AST      │       │  - GC       │       │  - Alloc    │
             │  - Visit    │       │  - Heap     │       │  - Panic    │
             └──────┬──────┘       └─────────────┘       └─────────────┘
                    │
        ┌───────────┼───────────┐
        │           │           │
 ┌──────▼──────┐ ┌──▼───────┐   │
 │  faxc-lex   │ │ faxc-par │   │
 │ (FRONTEND)  │ │(FRONTEND)│   │
 │ - Lexer     │ │ - Parser │   │
 │ - Token     │ │ - AST    │   │
 └──────┬──────┘ └───┬──────┘   │
        │            │          │
        │     ┌──────▼──────┐   │
        │     │  faxc-sem   │   │
        │     │ (FRONTEND)  │   │
        │     │ - HIR       │   │
        │     │ - Resolve   │   │
        │     │ - Check     │   │
        │     └──────┬──────┘   │
        │            │          │
        │     ┌──────▼──────┐   │
        │     │  faxc-mir   │   │
        │     │(MIDDLE-END) │   │
        │     │ - SSA       │   │
        │     │ - CFG       │   │
        │     └──────┬──────┘   │
        │            │          │
        │     ┌──────▼──────┐   │
        │     │  faxc-opt   │   │
        │     │(MIDDLE-END) │   │
        │     │ - Passes    │   │
        │     └──────┬──────┘   │
        │            │          │
        │     ┌──────▼──────┐   │
        │     │  faxc-lir   │   │
        │     │(MIDDLE-END) │   │
        │     │ - LIR       │   │
        │     │ - RegAlloc  │   │
        │     └──────┬──────┘   │
        │            │          │
        │     ┌──────▼──────┐   │
        │     │ faxc-codegen│   │
        │     │ (BACKEND)   │   │
        │     │ - LLVM      │   │
        │     └──────┬──────┘   │
        │            │          │
        │     ┌──────▼──────┐   │
        └────►│  faxc-link  │◄──┘
              │ (BACKEND)   │
              │ - Linker    │
              └──────┬──────┘
                     │
              ┌──────▼──────┐
              │  faxc-drv   │
              │  (DRIVER)   │
              │ - CLI       │
              │ - Pipeline  │
              └─────────────┘
```

### Dependency Rules

1. **Strict Layering**: Lower layers cannot depend on higher layers
2. **No Cross-Layer Dependencies**: Frontend → Middle-end → Backend is strictly one-way
3. **Core is Universal**: `faxc-util` and `faxc-err` can be depended on by anyone
4. **Driver Knows All**: Only `faxc-drv` can depend on all layers (orchestration)

---

## Development Strategy

### Milestones

| Milestone | Target | Description | Success Criteria |
|-----------|--------|-------------|------------------|
| **M0** | Week 2 | Foundation | `faxc-util`, `faxc-err`, `faxc-ast` complete |
| **M1** | Week 4 | Lexer | Source → Tokens for all valid programs |
| **M2** | Week 6 | Parser | Tokens → AST for all valid programs |
| **M3** | Week 8 | Frontend | AST → HIR with type checking **(Milestone B)** |
| **M4** | Week 10 | MIR | HIR → MIR with SSA form |
| **M5** | Week 12 | Optimizer | MIR optimization passes |
| **M6** | Week 14 | LIR | MIR → LIR with register allocation |
| **M7** | Week 16 | Codegen | LIR → LLVM IR → Object file |
| **M8** | Week 18 | Full Pipeline | Source → Executable |

### Parallel Development Teams

```
┌─────────────────────────────────────────────────────────────────┐
│                    TEAM STRUCTURE                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Core Infrastructure Team                                        │
│  ├── Responsibility: faxc-util, faxc-err, faxc-ast              │
│  ├── Timeline: Week 1-2                                         │
│  └── Deliverables: Foundation for all other teams               │
│                                                                  │
│  Frontend Team A                                                 │
│  ├── Responsibility: faxc-lex                                   │
│  ├── Timeline: Week 3-4                                         │
│  └── Deliverables: Working lexer                                │
│                                                                  │
│  Frontend Team B                                                 │
│  ├── Responsibility: faxc-par                                   │
│  ├── Timeline: Week 4-6                                         │
│  └── Deliverables: Working parser                               │
│                                                                  │
│  Frontend Team C                                                 │
│  ├── Responsibility: faxc-sem                                   │
│  ├── Timeline: Week 6-8                                         │
│  └── Deliverables: Working type checker                         │
│                                                                  │
│  Middle-end Team                                                 │
│  ├── Responsibility: faxc-mir, faxc-opt, faxc-lir               │
│  ├── Timeline: Week 7-14                                        │
│  └── Deliverables: Optimized LIR                                │
│                                                                  │
│  Backend Team                                                    │
│  ├── Responsibility: faxc-codegen, faxc-link                    │
│  ├── Timeline: Week 12-16                                       │
│  └── Deliverables: Working executable                           │
│                                                                  │
│  Runtime Team                                                    │
│  ├── Responsibility: fgc, faxc-rt                               │
│  ├── Timeline: Week 1-13                                        │
│  └── Deliverables: Working GC and runtime                       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Parallel Execution Timeline

```
Week:     1    2    3    4    5    6    7    8    9   10   11   12   13   14   15   16   17   18
          │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │
Core      [==== faxc-util/err/ast ====]
          │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │
Frontend       [=== lex ===][=== par ===][=== sem ===]
          │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │
Middle                                [====== mir ======][=== opt ===][=== lir ===]
          │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │
Backend                                           [====== codegen ======][=== link ===]
          │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │
Runtime [============ fgc ============][====== faxc-rt ======]
          │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │
Driver                                            [=========== faxc-drv integration ===========]
          │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │    │
Milestone                [LEX]      [PAR]      [SEM]      [MIR]      [LIR]      [FULL PIPELINE]
                                    ═══════════════════════════════
                                    MILESTONE B: Frontend Complete
```

---

## Technical Decisions

### ADR-001: Garbage Collection via FGC

**Context**: Memory management strategy for Fax.

**Decision**: Use concurrent garbage collector (FGC) instead of ownership/borrowing.

**Rationale**:
- Simpler programming model
- Easier for beginners
- Good enough performance for most use cases

**Consequences**:
- Runtime dependency (GC must be linked)
- Pause times (minimized by concurrent design)
- No compile-time memory safety guarantees

### ADR-002: LLVM Backend via Inkwell

**Context**: Code generation backend choice.

**Decision**: Use LLVM via Inkwell (safe Rust bindings).

**Rationale**:
- Multi-target support (x86, ARM, WASM)
- Mature optimization passes
- Industry standard
- Long-term maintainability

**Consequences**:
- LLVM dependency (~100MB)
- Longer compile times
- Excellent generated code quality

### ADR-003: Three-Layer IR (HIR/MIR/LIR)

**Context**: Intermediate representation design.

**Decision**: Three distinct IR levels with clear boundaries.

**Rationale**:
- HIR: Type-rich, close to source
- MIR: SSA form, optimization-friendly
- LIR: Machine-level, register allocation

**Consequences**:
- More code to maintain
- Clear separation of concerns
- Each layer can be optimized independently

### ADR-004: Strict Layering

**Context**: Module dependency management.

**Decision**: Enforce strict one-way dependencies: Frontend → Middle → Backend.

**Rationale**:
- Prevents circular dependencies
- Enables parallel development
- Clear architectural boundaries

**Consequences**:
- Requires careful API design
- May need shared types extracted

### ADR-005: Error Handling Pattern

**Context**: Error propagation across phases.

**Decision**: Use `Result<T, Vec<Diagnostic>>` pattern.

**Rationale**:
- Collect all errors before failing
- Consistent error format
- Easy to extend with suggestions

**Consequences**:
- More verbose than `?` operator
- Better user experience

---

## Quality Standards

### Code Quality

| Standard | Requirement |
|----------|-------------|
| **Clean Code** | Functions < 50 lines, single responsibility |
| **Naming** | Descriptive, consistent conventions |
| **DRY** | No duplication, extract common logic |
| **Comments** | Self-documenting code, doc comments for public APIs |
| **Testing** | 80%+ coverage for critical code |

### Documentation

| Type | Location | Standard |
|------|----------|----------|
| API Docs | In source (`///`) | All public items documented |
| Architecture | `docs/arch/` | Updated with major changes |
| Spec | `docs/spec/` | Complete language reference |
| Developer Guide | `docs/dev/` | Onboarding and workflows |

### Testing Strategy

```
┌─────────────────────────────────────────────────────────────────┐
│                      TESTING PYRAMID                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│                         ┌─────┐                                 │
│                        │ E2E │  ← Integration tests (10%)       │
│                       ├───────┤                                 │
│                      │ Integ │ ← Phase integration (20%)        │
│                     ├─────────┤                                 │
│                    │  Unit   │ ← Per-crate tests (70%)          │
│                   └───────────┘                                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Quality Gates

| Phase | Entry Criteria | Exit Criteria |
|-------|---------------|---------------|
| Lexer | M0 complete | 95% coverage, all tokens work |
| Parser | Lexer complete | 90% coverage, parses SPEC examples |
| Semantic | Parser complete | Type errors caught, HIR valid |
| MIR | Semantic complete | SSA form verified |
| LIR | MIR complete | Register allocated |
| Codegen | LIR complete | Executable produced |

---

## Appendix

### Related Documents

- [SPEC.md](../../SPEC.md) - Language specification
- [CONTRIBUTING.md](../../CONTRIBUTING.md) - Contribution guidelines
- [Pipeline](pipeline.md) - Detailed pipeline description
- [Technology](technology.md) - Technology stack

### Glossary

| Term | Definition |
|------|------------|
| **AST** | Abstract Syntax Tree |
| **HIR** | High-level Intermediate Representation |
| **MIR** | Mid-level Intermediate Representation |
| **LIR** | Low-level Intermediate Representation |
| **SSA** | Static Single Assignment |
| **CFG** | Control Flow Graph |
| **FGC** | Fax Garbage Collector |
| **DefId** | Definition ID (resolved name) |

---

*This document is maintained by the Fax Team. Last updated: 2026-02-17*
