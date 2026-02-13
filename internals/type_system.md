# Type System & Semantic Analysis Deep Dive

The Semantic Analyzer (Sema) is implemented in Haskell. It transforms an **Untyped AST** into a **Typed AST**.

## 1. Type Representation

Types are represented internally as a recursive algebraic data type:

```haskell
data Type 
    = TInt 
    | TFloat 
    | TBool 
    | TString 
    | TStruct String [(String, Type)] -- Name and Fields
    | TArray Type
    | TFunction [Type] Type           -- Args and Return
    | TVar Int                        -- For Type Inference (future)
    | TUnknown
```

## 2. Name Resolution & Scoping

Sema maintains a `SymbolTable` which is a stack of Maps (`[Map String Type]`).
- **Enter Scope**: Pushes a new empty Map onto the stack.
- **Exit Scope**: Pops the top Map.
- **Lookup**: Searches from the top of the stack to the bottom (handling shadowing).

## 3. The Unification Algorithm (Logic)

When Sema encounters an expression like `a + b`:
1. It recursively checks the type of `a`.
2. It recursively checks the type of `b`.
3. It ensures `type(a) == type(b)`.
4. It ensures the operator `+` is defined for that type.

## 4. Struct Validation

Structs are validated in two passes:
1. **Definition Pass**: Collects all struct names and field signatures to allow forward references.
2. **Body Pass**: Validates struct instantiations and field accesses (`obj.field`).

## 5. JSON Decoration

The output of Sema is the same AST structure but with an added `resolvedType` field for every expression node.

```json
{
  "type": "BinaryExpression",
  "left": { "type": "Literal", "value": 1, "resolvedType": "Int" },
  "operator": "+",
  "right": { "type": "Literal", "value": 2, "resolvedType": "Int" },
  "resolvedType": "Int"
}
```
