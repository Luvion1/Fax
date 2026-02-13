# Semantic Analyzer (Sema) Documentation

The Semantic Analyzer (Sema) is responsible for type checking and semantic analysis of the Fax programming language. It's implemented in **Haskell**.

## Overview

Sema performs:
- **Type Checking**: Ensure type safety
- **Type Inference**: Automatically deduce types
- **Scope Analysis**: Track variable definitions
- **Control Flow Analysis**: Detect missing returns, unreachable code
- **Pattern Matching**: Exhaustiveness checking

## Module Structure

```
src/Sema/
├── Diag.hs           # Base types (Diag, Severity, Type, SemanticError)
├── Types.hs           # Type definitions, inference, unification
├── Errors.hs         # Error types, suggestions
├── Pretty.hs         # Error formatting, diagnostics
├── ASTUtils.hs       # AST helper functions
├── Checker.hs        # Main type checking logic
├── ConstantFolding.hs # Constant expression evaluation
├── ControlFlow.hs    # Flow analysis, pattern matching
└── Main.hs           # Entry point
```

## Type System

### Primitive Types

```haskell
data Type = TI64      -- 64-bit integer
          | TBool     -- Boolean
          | TStr      -- String
          | TVoid     -- No value
          | TNull     -- Null
          | TUnk      -- Unknown (for inference)
          | TFn [Type] Type  -- Function
          | TStruct String   -- Struct
          | TArr Type        -- Array
          | TPtr Type       -- Pointer
          | TTup [Type]      -- Tuple
```

### Type Inference

```haskell
-- Unification
unify :: Type -> Type -> Either String Subst

-- Type application
applySubst :: Subst -> Type -> Type

-- Binary operation type inference
inferBin :: String -> Type -> Type -> Either String Type
```

## Error Types

### Errors (E001-E022)

| Code | Error | Description |
|------|-------|-------------|
| E001 | UndefinedSymbol | Variable not defined |
| E002 | TypeMismatch | Type mismatch |
| E003 | NotAFunction | Called non-function |
| E004 | ArgCountMismatch | Wrong argument count |
| E005 | ConditionMustBeBool | Condition not boolean |
| E006 | RangeBoundsMustBeI64 | Range bounds not i64 |
| E007 | ImmutableAssignment | Assignment to immutable |
| E008 | FieldNotFound | Struct field missing |
| E009 | StructNotFound | Struct not defined |
| E010 | NotAStruct | Not a struct type |
| E011 | NotAnArray | Not an array type |
| E012 | ArrayTypeMismatch | Array elements differ |
| E013 | DuplicateSymbol | Symbol redefinition |
| E014 | MissingField | Missing struct field |
| E015 | ExtraField | Extra struct field |
| E016 | ReturnTypeMismatch | Return type wrong |
| E017 | BreakOutsideLoop | Break outside loop |
| E018 | ContinueOutsideLoop | Continue outside loop |
| E019 | ArgTypeMismatch | Argument type wrong |
| E020 | IndexMustBeI64 | Array index not i64 |
| E021 | MissingReturn | Function may not return |
| E022 | NonExhaustivePattern | Pattern not exhaustive |

### Warnings (W001-W009)

| Code | Warning | Description |
|------|---------|-------------|
| W001 | UnusedVariable | Variable unused |
| W002 | UnreachableCode | Unreachable code |
| W003 | ShadowingWarning | Variable shadows outer |
| W004 | ConstantCondition | Always true/false |
| W005 | InfiniteLoopDetected | Infinite loop |
| W006 | PossibleMissingReturn | May not return |
| W007 | SuspiciousPattern | Suspicious code |
| W008 | UnreachableCodeAfter | Unreachable after return |
| W009 | RedundantPattern | Redundant pattern |

## Control Flow Analysis

### Termination Checking

```haskell
data TerminationStatus
    = AlwaysTerminates
    | MayTerminate
    | NeverTerminates
```

Detects:
- **Missing Return**: Function doesn't return on all paths
- **Unreachable Code**: Code after return/break/continue

### Pattern Exhaustiveness

```haskell
data Pattern
    = PWild       -- Wildcard (_)
    | PVar String -- Variable
    | PCon String -- Constructor
    | PInt Integer
    | PBool Bool
    | PChar Char
    | PNull
```

Checks:
- **Exhaustive Patterns**: All cases handled
- **Redundant Patterns**: Patterns that never match

## Suggestions System

Sema provides smart suggestions for common errors:

```haskell
getSuggestion :: SemanticError -> Maybe Suggestion
```

Example suggestions:
- "Use comparison operators (==, !=, <, >) to get a boolean"
- "Change 'let x' to 'let mut x' to make it mutable"
- "Add a 'return' statement at the end of function"

## Building Sema

### Prerequisites

- **GHC** 9.8+
- **Cabal** or **Stack**

### Build Commands

```bash
# Using GHC directly
cd faxc/packages/sema
ghc -o sema -isrc -isrc/Sema \
    src/Main.hs \
    src/Sema/Diag.hs \
    src/Sema/Types.hs \
    src/Sema/Errors.hs \
    src/Sema/Pretty.hs \
    src/Sema/Api.hs \
    src/Sema/Checker.hs \
    src/Sema/ConstantFolding.hs \
    src/Sema/ControlFlow.hs \
    src/Sema/ASTUtils.hs \
    -package parsec \
    -package containers

# Using Stack
cd faxc/packages/sema
stack build

# Using Cabal
cd faxc/packages/sema
cabal build
```

## Running Sema

### Command Line

```bash
./sema < input.json > output.json
```

### Input Format

```json
{
  "type": "Program",
  "body": [
    {
      "type": "FunctionDeclaration",
      "name": "main",
      "returnType": "void",
      "args": [],
      "body": [],
      "loc": {"line": 1, "col": 1}
    }
  ]
}
```

### Output

On success: Validated AST JSON
On error: Error messages to stderr, exit code 1

## Testing

```bash
# Test basic functionality
echo '{"type": "Program", "body": []}' | ./sema /dev/stdin

# Test error detection
./sema test_type_mismatch.json

# Run all test cases
make test
```

## Extension Points

### Adding New Error Types

1. Add to `Diag.hs`:
   ```haskell
   data SemanticError
       = NewErrorType String
       | ...
   ```

2. Add error info in `Diag.hs`:
   ```haskell
   getErrorInfo (NewErrorType s) = ("EXXX", "Error message: " ++ s)
   ```

3. Add suggestion in `Errors.hs`:
   ```haskell
   getSuggestion (NewErrorType s) = Just $ Suggestion { ... }
   ```

### Adding New Type Checks

1. Add checking function in `Checker.hs`
2. Call from appropriate context in `check`
3. Use `reportError` to emit errors

## Performance

- **Lines processed/sec**: ~3,000
- **Memory usage**: ~50MB
- **Type inference**: Sub-millisecond for most expressions
