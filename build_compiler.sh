#!/bin/bash
# Script untuk membangun seluruh compiler Fax-lang

set -e  # Keluar jika ada error

ROOT_DIR=$(pwd)
echo "Membangun compiler Fax-lang di $ROOT_DIR..."

# Fungsi untuk membangun komponen
build_component() {
    local dir=$1
    local build_cmd=$2
    local name=$3
    
    echo "Membangun $name..."
    cd "$ROOT_DIR/$dir"
    eval "$build_cmd"
    cd "$ROOT_DIR"
    echo "✓ $name berhasil dibangun"
}

# Bangun lexer (Rust)
build_component "faxc/packages/lexer" "cargo build --release" "Lexer"

# Bangun parser (Zig)
build_component "faxc/packages/parser" "zig build -Doptimize=ReleaseSafe" "Parser"

# Bangun semantic analyzer (Haskell)
echo "Membangun Semantic Analyzer..."
cd "$ROOT_DIR/faxc/packages/sema"
mkdir -p bin
ghc -dynamic -isrc -o bin/sema_bin src/Main.hs
cd "$ROOT_DIR"
echo "✓ Semantic Analyzer berhasil dibangun"

# Bangun optimizer (Rust)
build_component "faxc/packages/optimizer" "cargo build --release" "Optimizer"

# Bangun codegen (C++)
echo "Membangun Code Generator..."
cd "$ROOT_DIR/faxc/packages/codegen"
mkdir -p build
cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make -j$(nproc)
cd "$ROOT_DIR"
echo "✓ Code Generator berhasil dibangun"

# Bangun runtime (Zig)
build_component "faxc/packages/runtime" "zig build" "Runtime"

echo ""
echo "Semua komponen compiler berhasil dibangun!"
echo ""
echo "Untuk menguji compiler, jalankan:"
echo "  cd /root/Fax && ./run_pipeline.sh simple_test.fax"