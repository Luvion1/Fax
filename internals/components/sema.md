# Component: Semantic Analyzer (Haskell)

The "Sema" phase is the brain of the compiler. It performs type checking and ensures the program is logically sound.

## Responsibility
- **Type Inference**: Deducing types where they are not explicitly provided.
- **Name Resolution**: Ensuring variables and functions are defined before use.
- **Scoping**: Managing lexical scopes and shadowing.

## Why Haskell?
Semantic analysis is essentially a tree-transformation and validation problem. Haskell's functional nature and powerful pattern matching (`case ... of`) make it perfect for traversing the AST and maintaining a functional type-environment state.

## Error Reporting
Sema provides high-quality error messages inspired by Rust:
```text
Sema Error: Type mismatch at line 5
Expected: Int
Found: String
```

## Internal Flow
1. **Import**: Parses the JSON AST from the Zig stage.
2. **Analysis**: Traverses the tree while maintaining a `SymbolTable`.
3. **Decoration**: Adds `resolvedType` information to every node.
4. **Export**: Outputs a "Typed AST".
