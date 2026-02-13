---
sidebar_position: 3
---

# Parser

**Location**: `faxc/packages/parser/`

The Parser is implemented in **Zig 0.14.1** and builds the Abstract Syntax Tree (AST).

## Features

- **AST Generation**: Builds tree structure from tokens
- **Statement Parsing**: Functions, control flow, declarations
- **Expression Parsing**: Operators, literals, identifiers
- **Error Reporting**: Clear error messages with location

## AST Nodes

```json
{
  "type": "FunctionDeclaration",
  "name": "main",
  "returnType": "void",
  "args": [],
  "body": [...]
}
```

## Build

```bash
cd faxc/packages/parser
zig build -Doptimize=ReleaseSafe
```

## Key Files

- `src/parser/parser.zig` - Main parser
- `src/parser/stmt.zig` - Statement parsing
- `src/parser/expr.zig` - Expression parsing
