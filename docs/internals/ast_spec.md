# AST Specification

This document defines the structure of the Abstract Syntax Tree (AST) used as the Intermediate Representation (IR) in Fax-lang.

## Root Node

The root of every AST file is a `Program` node.

```json
{
  "type": "Program",
  "body": [ ...Nodes... ]
}
```

## Common Node Structures

### Variable Declaration
```json
{
  "type": "VariableDeclaration",
  "name": "x",
  "kind": "let",
  "init": { "type": "Literal", "value": 42, "rawType": "int" },
  "resolvedType": "int"
}
```

### Function Declaration
```json
{
  "type": "FunctionDeclaration",
  "name": "add",
  "params": [
    { "name": "a", "type": "int" },
    { "name": "b", "type": "int" }
  ],
  "returnType": "int",
  "body": [ ...Statements... ]
}
```

### Binary Expression
```json
{
  "type": "BinaryExpression",
  "operator": "+",
  "left": { ...Node... },
  "right": { ...Node... },
  "resolvedType": "int"
}
```

## Type System Integration

During the `Sema` (Haskell) phase, every expression node is decorated with a `resolvedType` field. The `Codegen` (C++) phase relies on this field to determine which LLVM instructions to emit (e.g., `add i64` vs `fadd double`).

## Structs and Classes

Structs define a memory layout. The AST stores field offsets and types which are used by Fgc for root scanning.

```json
{
  "type": "StructDeclaration",
  "name": "Point",
  "fields": [
    { "name": "x", "type": "int" },
    { "name": "y", "type": "int" }
  ]
}
```
