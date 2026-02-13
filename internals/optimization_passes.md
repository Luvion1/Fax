# Optimization Passes (Python)

The Optimizer is a Python-based component that performs AST-to-AST transformations. It sits between `Sema` and `Codegen`.

## 1. Why Python?

Optimization often involves complex graph traversals and pattern matching. Python's rich ecosystem and ease of manipulation for JSON/Dict structures make it ideal for rapid prototyping of optimization rules.

## 2. Implemented Passes

### A. Constant Folding
Reduces constant expressions at compile time.
- Input: `1 + 2`
- Output: `3`
- Status: **Active**

### B. Dead Code Elimination (DCE)
Removes code that is unreachable or has no side effects.
- If an `if` condition is a constant `false`, the entire block is removed.
- Functions that are never called (and not exported) are removed.
- Status: **Partial**

### C. Strength Reduction
Replaces expensive operations with cheaper ones.
- `x * 2` becomes `x << 1`.
- Status: **Planned**

## 3. Data Flow Analysis

The optimizer builds a basic **Control Flow Graph (CFG)** for each function to track variable usage. This allows it to identify variables that are assigned but never read.

## 4. Usage in Pipeline

The Hub invokes the optimizer as follows:
```bash
python3 src/components/optimizer/optimizer.py input_ast.json
```
If the `--no-opt` flag is passed to the Hub, this stage is skipped to speed up debug builds.
