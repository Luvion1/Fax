#!/bin/bash

# Fax Compiler Pipeline
# Usage: ./run_pipeline.sh <input_file.fax>

INPUT=$1
if [ -z "$INPUT" ]; then
    echo "Usage: $0 <input.fax>"
    exit 1
fi

# Determine script directory
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_LEX="$ROOT/faxc/packages/lexer/target/release/lexer"
BIN_PARSE="$ROOT/faxc/packages/parser/zig-out/bin/parser"
BIN_SEMA="$ROOT/faxc/packages/sema/bin/sema_bin"
BIN_OPT="$ROOT/faxc/packages/optimizer/target/release/fax-opt"
BIN_CG="$ROOT/faxc/packages/codegen/build/faxc_cpp"
LIB_RT="$ROOT/faxc/packages/runtime/zig-out/lib/libfaxruntime.a"

# Temporary files
TOKENS="temp_tokens.json"
AST="temp_ast.json"
TYPED="temp_typed.json"
OPT="temp_optimized.json"
LLVM="output.ll"
OUT="output"

echo "[Fax] Processing $INPUT..."

# 1. Lexer
$BIN_LEX "$INPUT" > "$TOKENS" || { echo "Lexer failed"; exit 1; }

# 2. Parser
$BIN_PARSE "$TOKENS" > "$AST" || { echo "Parser failed"; exit 1; }

# 3. Semantic Analysis
$BIN_SEMA "$AST" > "$TYPED" || { echo "Sema failed"; exit 1; }

# 4. Optimization
$BIN_OPT "$TYPED" --opt-level=intermediate > "$OPT" || { echo "Optimizer failed"; exit 1; }

# 5. Code Generation
$BIN_CG "$OPT" > "$LLVM" || { echo "Codegen failed"; exit 1; }

# 6. Linking
zig cc "$LLVM" "$LIB_RT" -o "$OUT" -Wno-override-module -lc -O2 -g || { echo "Linking failed"; exit 1; }

echo "[Fax] Successfully built: $OUT"

# Cleanup
rm -f "$TOKENS" "$AST" "$TYPED" "$OPT"
