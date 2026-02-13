#!/bin/bash

# Build all Fax-lang compiler components

ROOT="/root/Fax"

echo "Building Lexer (Rust)..."
(cd "$ROOT/faxc/packages/lexer" && cargo build --release)

echo "Building Parser (Zig)..."
(cd "$ROOT/faxc/packages/parser" && zig build)

echo "Building Optimizer (Rust)..."
(cd "$ROOT/faxc/packages/optimizer" && cargo build --release)

echo "Building Codegen (C++)..."
(cd "$ROOT/faxc/packages/codegen/build" && cmake .. && make)

echo "Building Runtime (Zig)..."
(cd "$ROOT/faxc/packages/runtime" && zig build)

echo "Building Semantic Analyzer (Haskell)..."
(cd "$ROOT/faxc/packages/sema" && ghc -isrc src/Main.hs -o bin/sema_bin && chmod +x bin/sema_bin)

echo "Building Hub (TypeScript)..."
(cd "$ROOT/faxc/packages/hub" && npm install && npm run build)

echo "All components built successfully!"