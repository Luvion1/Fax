# Component: Lexer (Rust)

The Lexer is the first stage of the Fax compiler. It is written in Rust for its exceptional performance and robust UTF-8 string handling.

## Responsibility
- Convert raw source text into a stream of JSON-serialized tokens.
- Handle escaped characters and multi-line strings.
- Track source locations (line/column) for error reporting.

## Implementation Details
- **Location**: `faxc/src/components/lexer/lexer.rs`
- **Engine**: A manual state machine for maximum speed.
- **Output Format**:
```json
[
  { "type": "Keyword", "value": "fn", "line": 1, "col": 1 },
  { "type": "Identifier", "value": "main", "line": 1, "col": 4 }
]
```

## Why Rust?
Rust was chosen for the Lexer because tokenization is an I/O and CPU-bound task. Rust's `regex` and `serde` libraries allow for extremely fast serialization to JSON, which is the bottleneck in our polyglot pipeline.
