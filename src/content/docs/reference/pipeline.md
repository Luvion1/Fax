---
title: Compiler Pipeline
description: Understanding the Fax compiler pipeline
---

## Pipeline Stages

The Fax compiler processes source code through multiple stages:

### 1. Lexical Analysis

Converts source code into tokens:

```
Input:  let x = 10;
Output: [Let, Identifier("x"), Equals, Number("10"), Semicolon]
```

**Key features**:
- String literals with escape sequences
- Comments (single-line and multi-line)
- Error recovery with line/column info

### 2. Parsing

Builds Abstract Syntax Tree (AST) from tokens:

```json
{
  "type": "Program",
  "body": [
    {
      "type": "VariableDeclaration",
      "name": "x",
      "value": {
        "type": "NumberLiteral",
        "value": "10"
      }
    }
  ]
}
```

### 3. Semantic Analysis

Validates AST:
- Type checking
- Undefined symbol detection
- Control flow analysis
- Pattern exhaustiveness checking

### 4. Optimization

Transforms AST for better performance:
- Constant folding
- Dead code elimination
- Common subexpression elimination

### 5. Code Generation

Generates LLVM IR:

```llvm
; Variable declaration
%x_ptr = alloca i64
store i64 10, i64* %x_ptr
```

### 6. Runtime Execution

Executes compiled code with FGC.

## Data Flow

Each stage communicates via JSON:

```
Source → Tokens (JSON) → AST (JSON) → Validated AST (JSON) → LLVM IR → Binary
```

## Pipeline Orchestration

The pipeline is orchestrated by the `hub` component (TypeScript):

```typescript
// Simplified pipeline
const pipeline = new PipelineBuilder()
  .addStage('lexer', lexerAdapter)
  .addStage('parser', parserAdapter)
  .addStage('sema', semaAdapter)
  .addStage('optimizer', optimizerAdapter)
  .addStage('codegen', codegenAdapter)
  .build();

const result = await pipeline.execute(sourceCode);
```
