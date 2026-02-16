#!/bin/bash

# Script to generate C++ code from protobuf definitions

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROTO_DIR="$SCRIPT_DIR/proto"
GEN_DIR="$SCRIPT_DIR/faxc/Fax/Compiler/Proto/Generated"

echo "Generating protobuf C++ code..."

# Create output directory
mkdir -p "$GEN_DIR"

# Find all proto files
PROTO_FILES=(
  "$PROTO_DIR/common.proto"
  "$PROTO_DIR/types.proto"
  "$PROTO_DIR/literal.proto"
  "$PROTO_DIR/pattern.proto"
  "$PROTO_DIR/token.proto"
  "$PROTO_DIR/expr.proto"
  "$PROTO_DIR/decl.proto"
  "$PROTO_DIR/compiler.proto"
)

# Generate C++ code
protoc \
  --cpp_out="$GEN_DIR" \
  --proto_path="$PROTO_DIR" \
  "${PROTO_FILES[@]}"

echo "Generated C++ files in $GEN_DIR"

# List generated files
echo "Generated files:"
ls -la "$GEN_DIR"/*.cc "$GEN_DIR"/*.h 2>/dev/null || ls -la "$GEN_DIR"/*.cpp "$GEN_DIR"/*.h 2>/dev/null
