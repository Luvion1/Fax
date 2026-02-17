# Architecture Overview

## Design Principles

1. **Modularity**: Each compiler phase is an independent crate
2. **Testability**: Comprehensive test coverage at all levels
3. **Performance**: Zero-cost abstractions, optimized algorithms
4. **Safety**: Memory safety through Rust + GC integration
5. **Ergonomics**: Clear error messages, fast compile times

## Crate Dependencies

```
faxc-drv (driver)
    ├── faxc-lex (lexer)
    ├── faxc-par (parser)
    ├── faxc-sem (semantic)
    ├── faxc-mir (mid-level)
    ├── faxc-lir (low-level)
    ├── faxc-gen (codegen)
    └── faxc-util (utilities)

fgc (garbage collector)
    └── faxc-util

faxc-sem depends on fgc for type-level GC integration
```

## Data Structures

### Tokens (faxc-lex)
```rust
pub struct Token {
    kind: TokenKind,
    span: Span,
}
```

### AST (faxc-par)
```rust
pub enum Expr {
    Literal(Literal),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Identifier(Ident),
}
```

### HIR (faxc-sem)
```rust
pub struct Hir {
    items: Vec<Item>,
    types: TypeContext,
}
```

### MIR (faxc-mir)
```rust
pub struct Mir {
    functions: Vec<Function>,
    basic_blocks: Vec<BasicBlock>,
}
```

## Error Handling Strategy

- **Non-recoverable**: Panic (bugs only)
- **Recoverable**: Diagnostic system
- **Propagation**: anyhow for driver, thiserror for crates

## Concurrency Model

- **Compiler**: Single-threaded per translation unit
- **Parallel**: Multiple files compiled concurrently
- **Runtime**: M:N threading with work-stealing