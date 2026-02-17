# Fax Compiler Pipeline

## Overview

The Fax compiler follows a multi-phase compilation pipeline designed for enterprise-scale development with static typing and garbage collection support.

## Compilation Stages

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              SOURCE INPUT                                    │
│                              (.fax files)                                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PHASE 1: LEX                                    │
│                           Tokenizer (faxc-lex)                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                                   │
│  │  scan.rs │─▶│  tok.rs  │─▶│ srcmap.rs│                                   │
│  └──────────┘  └──────────┘  └──────────┘                                   │
│       │                                                          │         │
│       ▼                                                          ▼         │
│  Characters ────────────────────────────────────────────────▶  Tokens       │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PHASE 2: PAR                                    │
│                          Parser (faxc-par)                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                                   │
│  │  prs.rs  │─▶│  syn.rs  │─▶│  prec.rs │                                   │
│  └──────────┘  └──────────┘  └──────────┘                                   │
│       │                                             │                       │
│       ▼                                             ▼                       │
│   Tokens  ──────────────────────────────────────▶   AST                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PHASE 3: SEM                                    │
│                      Semantic Analysis (faxc-sem)                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐                     │
│  │  res.rs  │─▶│  chk.rs  │─▶│  inf.rs  │─▶│  ty.rs   │                     │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘                     │
│       │          │                                            │             │
│       ▼          ▼                                            ▼             │
│  Name Res  ──▶ Type Check ────────────────────────────────▶   HIR           │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PHASE 4: MIR                                    │
│                    Mid-Level IR (faxc-mir)                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                                   │
│  │ build.rs │─▶│  ssa.rs  │─▶│  opt/    │                                   │
│  └──────────┘  └──────────┘  └──────────┘                                   │
│       │             │             │                                         │
│       ▼             ▼             ▼                                         │
│    HIR   ─────▶  SSA Form  ──▶  Optimized MIR                               │
│                               (cst, dce, inl, licm)                         │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PHASE 5: LIR                                    │
│                     Low-Level IR (faxc-lir)                                 │
│  ┌──────────┐  ┌──────────┐                                                 │
│  │  reg.rs  │─▶│ instr.rs │                                                 │
│  └──────────┘  └──────────┘                                                 │
│       │             │                                                       │
│       ▼             ▼                                                       │
│   Reg Alloc  ─▶  Instruction Selection ─────────────────▶  LIR              │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PHASE 6: GEN                                    │
│                      Code Generation (faxc-gen)                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐                     │
│  │ llvm.rs  │─▶│  asm.rs  │─▶│  obj.rs  │─▶│ link.rs  │                     │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘                     │
│       │             │             │             │                           │
│       ▼             ▼             ▼             ▼                           │
│    LLVM IR  ─▶  Assembly  ─▶  Object  ──▶  Executable                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Phase Details

### Phase 1: Lexical Analysis (faxc-lex)
- **scan.rs**: Character stream scanning
- **tok.rs**: Token definitions and types
- **srcmap.rs**: Source location mapping for diagnostics
- **diag.rs**: Lexer-specific error reporting

### Phase 2: Parsing (faxc-par)
- **prs.rs**: Recursive descent parser implementation
- **syn.rs**: Abstract Syntax Tree (AST) definitions
- **prec.rs**: Operator precedence handling
- **expr.rs**: Expression parsing
- **item.rs**: Item/declaration parsing

### Phase 3: Semantic Analysis (faxc-sem)
- **res.rs**: Name resolution and scoping
- **chk.rs**: Type checking engine
- **inf.rs**: Type inference (Hindley-Milner)
- **ty.rs**: Type system definitions
- **hir.rs**: High-level Intermediate Representation

### Phase 4: MIR Optimization (faxc-mir)
- **build.rs**: HIR to MIR lowering
- **ssa.rs**: Static Single Assignment form
- **opt/**: Optimization passes
  - **cst.rs**: Constant folding
  - **dce.rs**: Dead code elimination
  - **inl.rs**: Function inlining
  - **licm.rs**: Loop invariant code motion
- **pass.rs**: Pass manager

### Phase 5: LIR Generation (faxc-lir)
- **reg.rs**: Register allocation (graph coloring)
- **instr.rs**: Instruction selection

### Phase 6: Code Generation (faxc-gen)
- **llvm.rs**: LLVM IR generation
- **asm.rs**: Assembly generation
- **obj.rs**: Object file output
- **link.rs**: Linking phase

## GC Integration (fgc)

The Fax Garbage Collector (FGC) integrates across multiple phases:

```
SEM Phase: Type system tracks GC-managed types
    │
    ▼
MIR Phase: Insert write barriers and GC roots
    │
    ▼
GEN Phase: Generate GC metadata and runtime calls
```

### FGC Components
- **gc.rs**: Core GC algorithm
- **alloc.rs**: Memory allocation
- **mark.rs**: Mark phase (tri-color)
- **sweep.rs**: Sweep phase
- **gen.rs**: Generational collection
- **conc.rs**: Concurrent collection
- **write.rs**: Write barriers
- **heap.rs**: Heap management

## Pipeline Data Flow

```
Source Code
    │
    ├──▶ [Lex] ──▶ Token Stream
    │
    ├──▶ [Par] ──▶ AST
    │
    ├──▶ [Sem] ──▶ HIR + Type Info
    │                    │
    │                    ├──▶ GC Type Analysis
    │                    │
    ├──▶ [MIR] ──▶ SSA-MIR
    │       │
    │       ├──▶ GC Write Barriers
    │       │
    │       ├──▶ Optimizations
    │       │
    ├──▶ [LIR] ──▶ Target-specific IR
    │       │
    │       ├──▶ Register Allocation
    │       │
    ├──▶ [Gen] ──▶ Machine Code
    │
    ▼
Executable
```

## Error Handling

Each phase reports diagnostics through the unified diagnostic system:
- **Level**: Error, Warning, Note
- **Location**: File, line, column
- **Context**: Source snippet with highlighting
- **Help**: Suggested fixes when available

## Incremental Compilation

The pipeline supports incremental compilation through:
- Fine-grained dependency tracking
- Serialized intermediate representations
- Change detection at AST level
- Selective recompilation