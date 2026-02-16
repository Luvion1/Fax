# Fax Compiler Module Structure

This document describes the modular organization of the Fax Compiler codebase following best practices for Lean 4 projects.

## Overview

The Fax Compiler is organized into a hierarchical module structure with clear separation of concerns:

```
faxc/
├── Fax.lean              # Main entry point - exports all modules
├── StdLib.lean           # Standard library
└── Compiler/
    ├── AST.lean          # Abstract Syntax Tree definitions
    ├── Lexer.lean        # Lexical analysis
    ├── Parser.lean       # Syntax analysis
    ├── Semantic.lean     # Semantic analysis (type checking)
    ├── Codegen.lean      # Code generation (LLVM IR)
    ├── Driver.lean       # Compiler driver & CLI
    ├── Proto.lean        # Protocol Buffers & gRPC
    ├── Runtime.lean      # FGC (Garbage Collector)
    └── Validation.lean   # Input validation
```

## Module Organization Principles

### 1. Hierarchical Structure

Each major component has its own directory with submodules:

```
Compiler/Semantic/
├── Semantic.lean       # Main entry - exports all submodules
├── Types.lean          # Type definitions
├── Scope.lean          # Scope management
├── Inference.lean      # Type inference
├── Errors.lean         # Error types
├── Checker.lean        # Type checker implementation
└── Proto.lean          # Protobuf integration
```

### 2. Index File Pattern

Each module directory has an index file at the parent level (e.g., `Compiler/Semantic.lean`) that:
- Imports all submodules
- Re-exports public APIs
- Provides module-level documentation

Example:
```lean
/-
Semantic Analysis Module - Main Entry Point
Exports all semantic analysis components
-/

import Compiler.Semantic.Types
import Compiler.Semantic.Checker
import Compiler.Semantic.Errors

namespace Compiler.Semantic

export Types (Symbol SymbolTable)
export Checker (TypeChecker typeCheckModule)
export Errors (SemanticError)

end Compiler.Semantic
```

### 3. Submodule Organization

Each submodule focuses on a specific responsibility:

**Semantic Analysis:**
- `Types.lean` - Type system definitions
- `Scope.lean` - Scope and symbol management
- `Inference.lean` - Hindley-Milner type inference
- `Checker.lean` - Main type checking logic
- `Errors.lean` - Error types and reporting
- `Proto.lean` - Protobuf service integration

**Validation:**
- `Core.lean` - Validation result types and operations
- `Source.lean` - Source code validation
- `Identifiers.lean` - Identifier name validation
- `Types.lean` - Type name validation
- `Limits.lean` - Limits and constraints validation

### 4. Export Conventions

Each index file exports:
- **Types** - Public type definitions
- **Functions** - Main API functions
- **Instances** - Type class instances (when needed)

Use explicit exports rather than wildcard exports for better documentation:
```lean
export Types (Symbol SymbolKind SymbolTable)
export Checker (TypeChecker typeCheckModule checkExpr)
```

## Module Dependencies

### Dependency Graph

```
Fax.lean
├── Compiler.AST
│   ├── Types, Patterns, Exprs, Stmts, Decls
├── Compiler.Lexer
│   ├── Tokens, State, Helpers
├── Compiler.Parser
│   ├── Lexer, AST, Types, Exprs, Stmts, Decls
├── Compiler.Semantic
│   ├── AST, Types, Scope, Inference, Errors, Checker
├── Compiler.Codegen
│   ├── AST, Types, IR, Expr, Stmt
├── Compiler.Driver
│   ├── IO, Proto, Semantic, Codegen
├── Compiler.Proto
│   ├── Messages, Wire, Codec, Services, Grpc
├── Compiler.Runtime
│   ├── GC (ZPointer, Barrier, Heap, Mark, etc.)
└── Compiler.Validation
    ├── Core, Source, Identifiers, Types, Limits
```

### Import Guidelines

1. **Always use full paths** for imports:
   ```lean
   import Compiler.AST.Types
   import Compiler.Semantic.Errors
   ```

2. **Minimize dependencies** - Import only what you need

3. **Avoid circular dependencies** - Use forward declarations or restructure

4. **Open namespaces selectively**:
   ```lean
   open Compiler.AST
   open Compiler.Semantic.Types
   ```

## File Naming Conventions

1. **Module index files** match the directory name:
   - `Compiler/Semantic.lean` for `Compiler/Semantic/` directory

2. **Submodule files** use PascalCase:
   - `Checker.lean`, `Inference.lean`, `Validation.lean`

3. **Helper modules** use descriptive names:
   - `Core.lean`, `Utils.lean`, `Helpers.lean`

4. **Type definitions** in `Types.lean` files

## Documentation Standards

### File Headers

Every Lean file should have a documentation header:

```lean
/-
Module Name - Brief Description

Longer description of what this module does.

Key Features:
- Feature 1
- Feature 2
- Feature 3
-/
```

### Module Documentation

Index files should include:
- Purpose of the module
- List of submodules
- Key exports
- Usage examples (when helpful)

### Function Documentation

Public functions should have docstrings:

```lean
/--
Type check a module.

Returns a `SemanticResult` containing any errors found and the symbol table.
-/def typeCheckModule (module : Module) : SemanticResult := ...
```

## Best Practices

### 1. Keep Modules Focused

Each file should have a single responsibility:
- ❌ Don't mix type definitions with complex logic
- ✅ Separate types (`Types.lean`) from logic (`Checker.lean`)

### 2. Use Namespaces

All code should be in a namespace:
```lean
namespace Compiler.Semantic.Checker

-- implementation here

end Compiler.Semantic.Checker
```

### 3. Explicit Exports

Be explicit about what you export:
```lean
-- Good
export Types (Symbol SymbolTable)

-- Avoid
export Types  -- exports everything
```

### 4. Avoid Deep Nesting

Maximum recommended nesting: 3-4 levels
- `Compiler/Semantic/Checker.lean` ✓
- `Compiler/Semantic/Checker/Expressions/Operators.lean` ✗

### 5. Consistent Naming

- **Types**: PascalCase (`TypeChecker`, `SemanticError`)
- **Functions**: camelCase (`typeCheckModule`, `inferExpr`)
- **Modules**: PascalCase matching file name

## Migration Guide

When reorganizing code:

1. Create new module structure
2. Move code to appropriate files
3. Update imports in moved files
4. Create index file with exports
5. Update parent module imports
6. Update documentation
7. Test compilation

### Example: Moving Code

Before:
```
Compiler/
├── Semantic.lean    (830 lines - too large)
```

After:
```
Compiler/
├── Semantic.lean    (30 lines - index only)
└── Semantic/
    ├── Types.lean   (150 lines)
    ├── Scope.lean   (100 lines)
    ├── Inference.lean (200 lines)
    ├── Checker.lean (300 lines)
    ├── Errors.lean  (100 lines)
    └── Proto.lean   (100 lines)
```

## Validation Module Example

The Validation module demonstrates best practices:

```
Compiler/Validation/
├── Core.lean        # ValidationResult type and operations
├── Source.lean      # Source code validation
├── Identifiers.lean # Identifier validation
├── Types.lean       # Type name validation
└── Limits.lean      # Constraint validation
```

Each file:
- Has a single responsibility
- Imports only what it needs
- Exports specific functions
- Is well-documented

The index file (`Compiler/Validation.lean`) provides a unified interface:
```lean
export Core (ValidationResult ok errors isOk)
export Source (validateSourceCode)
export Identifiers (validateIdentifier)
export Types (validateTypeName)
export Limits (validateIntLiteral validateStringLiteral)
```

## Proto Module Example

The Proto module handles Protocol Buffer and gRPC services:

```
Compiler/Proto/
├── Messages.lean      # Protocol buffer message types
├── Wire.lean          # Wire format encoding/decoding
├── Binary.lean        # Binary serialization
├── Codec.lean         # Encoding/decoding (index)
├── Codec/
│   ├── Token.lean    # Token stream encoding
│   ├── Types.lean    # Type encoding
│   └── AST.lean      # AST encoding
├── Converters.lean    # AST <-> Protobuf converters (index)
├── Converters/
│   ├── Token.lean
│   ├── Types.lean
│   ├── Pattern.lean
│   ├── Expr.lean
│   └── Decl.lean
├── Services.lean      # Service definitions
├── Grpc.lean          # gRPC client
├── GrpcCodegen.lean   # gRPC code generation
├── Server.lean        # gRPC server
├── Discovery.lean     # Service discovery
├── Cache.lean         # Result caching
├── Diagnostics.lean   # Error reporting
├── Semantic.lean      # Semantic analysis
└── Test.lean          # Test utilities
```

Key aspects:
- **Services** defined in `Services.lean` (not subdirectory)
- **Discovery** defines `ServiceEndpoint` to avoid circular dependencies
- **Codec** and **Converters** have subdirectories with index files
- All gRPC components are at top level (Grpc, GrpcCodegen, Server)

## Future Improvements

1. **Add lakefile.lean** for proper Lean 4 project configuration
2. **Add tests** for each module
3. **Add benchmarks** for performance-critical modules
4. **Generate documentation** with `doc-gen4`
5. **Add CI checks** for import organization

## References

- [Lean 4 Manual](https://lean-lang.org/lean4/doc/)
- [Lean Prover Style Guide](https://leanprover-community.github.io/contribute/style.html)
- [Lean 4 Best Practices](https://leanprover-community.github.io/mathlib4_docs/docs/lean4/quickstart.html)
