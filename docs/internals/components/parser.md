# Component: Parser (Zig)

The Parser converts the token stream into a hierarchical Abstract Syntax Tree (AST). It is implemented in Zig (0.15.2).

## Responsibility
- Validate the grammatical structure of the program.
- Construct a hierarchical JSON AST.
- Handle operator precedence and associativity.

## Recursive Descent Strategy
The parser uses a **Recursive Descent** approach. Each grammatical rule is mapped to a Zig function:
- `parseExpression()`
- `parseStatement()`
- `parseFunction()`

## Memory Management
Unlike the Lexer, the Parser deals with complex tree structures. Zig's `ArenaAllocator` is used to manage AST nodes efficiently. Once the JSON is emitted to `stdout`, the entire arena is wiped, ensuring zero memory leaks between compilation steps.

## AST Layout
The Zig parser emits a nested JSON structure that represents the program's logic.
```json
{
  "type": "FunctionDeclaration",
  "name": "main",
  "body": [ ... ]
}
```
