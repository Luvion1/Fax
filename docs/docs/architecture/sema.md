---
sidebar_position: 4
---

# Sema (Semantic Analyzer)

**Location**: `faxc/packages/sema/`

The Semantic Analyzer is implemented in **Haskell (GHC 9.6.6)** and performs type checking.

## Features

- **Type Checking**: Validate types
- **Type Inference**: Automatically deduce types
- **Scope Analysis**: Track variable definitions
- **Control Flow Analysis**: Missing returns, unreachable code
- **Pattern Matching**: Exhaustiveness checking
- **Error Suggestions**: Smart error messages

## Error Types

### Errors (E001-E022)

| Code | Description |
|------|-------------|
| E001 | Undefined symbol |
| E002 | Type mismatch |
| E003 | Not a function |
| E021 | Missing return |

### Warnings (W001-W009)

| Code | Description |
|------|-------------|
| W001 | Unused variable |
| W008 | Unreachable code |

## Modules

| Module | Description |
|--------|-------------|
| `Types.hs` | Type definitions, inference |
| `Checker.hs` | Main type checking |
| `ControlFlow.hs` | Flow analysis |

## Build

```bash
cd faxc/packages/sema
ghc -o sema src/Main.hs
```
