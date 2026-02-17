# Fax Language Specification

## Overview

Fax is a statically typed systems programming language with garbage collection, designed for performance-critical applications.

## Grammar

### Lexical Elements

```ebnf
identifier = letter { letter | digit | "_" }
number = digit { digit }
string = '"' { character } '"'
```

### Types

```ebnf
type = primitive_type | composite_type | function_type
primitive_type = "int" | "float" | "bool" | "string"
composite_type = "[" type "]" | "(" { type ", " } ")" | identifier
function_type = "fn" "(" { type ", " } ")" [ "->" type ]
```

### Expressions

```ebnf
expression = primary | binary_expr | unary_expr | call_expr
primary = identifier | number | string | "(" expression ")"
binary_expr = expression operator expression
unary_expr = operator expression
call_expr = expression "(" { expression ", " } ")"
operator = "+" | "-" | "*" | "/" | "==" | "!=" | "<" | ">"
```

### Statements

```ebnf
statement = expression_stmt | let_stmt | return_stmt | block_stmt
let_stmt = "let" identifier [ ":" type ] "=" expression
return_stmt = "return" [ expression ]
block_stmt = "{" { statement } "}"
```

### Declarations

```ebnf
declaration = function_decl | struct_decl | enum_decl
function_decl = "fn" identifier "(" parameters ")" [ "->" type ] block_stmt
struct_decl = "struct" identifier "{" { field ", " } "}"
enum_decl = "enum" identifier "{" { variant ", " } "}"
```