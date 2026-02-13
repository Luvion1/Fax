---
sidebar_position: 2
---

# Lexer

**Location**: `faxc/packages/lexer/`

The Lexer is implemented in **Rust 1.93.0** and handles tokenization of the source code.

## Features

- **36 Token Types**: Keywords, operators, literals
- **85+ Keywords**: Full language keywords
- **Error Recovery**: Handles malformed input gracefully
- **JSON Output**: Outputs tokens as JSON

## Output Format

```json
[
  {"type": "Keyword", "value": "fn", "loc": {"line": 1, "col": 1}},
  {"type": "Identifier", "value": "main", "loc": {"line": 1, "col": 4}},
  {"type": "LParen", "value": "(", "loc": {"line": 1, "col": 9}}
]
```

## Build

```bash
cd faxc/packages/lexer
cargo build --release
```

## Key Files

- `src/lexer/tokenizer.rs` - Main tokenizer
- `src/lexer/token.rs` - Token definitions
