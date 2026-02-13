#!/bin/bash
# Script untuk menginstal dependensi yang diperlukan untuk Fax-lang

echo "Memeriksa dependensi untuk Fax-lang..."

# Fungsi untuk memeriksa apakah program terinstal
check_installed() {
    if command -v "$1" &> /dev/null; then
        echo "✓ $1 sudah terinstal"
        return 0
    else
        echo "✗ $1 belum terinstal"
        return 1
    fi
}

# Periksa Rust
if ! check_installed "cargo"; then
    echo "Menginstal Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    echo "✓ Rust berhasil diinstal"
else
    echo "Menggunakan Rust yang sudah terinstal"
fi

# Periksa Zig
if ! check_installed "zig"; then
    echo "Menginstal Zig..."
    # Deteksi arsitektur
    ARCH=$(uname -m)
    if [ "$ARCH" = "x86_64" ]; then
        ZIG_VERSION="0.13.0"
        wget https://ziglang.org/download/$ZIG_VERSION/zig-linux-x86_64-$ZIG_VERSION.tar.xz
        tar -xf zig-linux-x86_64-$ZIG_VERSION.tar.xz
        sudo mv zig-linux-x86_64-$ZIG_VERSION /opt/zig
        sudo ln -sf /opt/zig/zig /usr/local/bin/zig
        rm zig-linux-x86_64-$ZIG_VERSION.tar.xz
        echo "✓ Zig berhasil diinstal"
    else
        echo "⚠ Arsitektur $ARCH belum didukung dalam skrip ini. Silakan instal Zig secara manual."
    fi
else
    echo "Menggunakan Zig yang sudah terinstal"
fi

# Periksa GHC (Haskell)
if ! check_installed "ghc"; then
    echo "Menginstal GHC (Haskell)..."
    # Di Ubuntu/Debian
    if command -v apt &> /dev/null; then
        sudo apt update
        sudo apt install -y ghc
    # Di CentOS/RHEL/Fedora
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y ghc
    # Di Arch Linux
    elif command -v pacman &> /dev/null; then
        sudo pacman -S ghc
    else
        echo "⚠ Tidak dapat menginstal GHC secara otomatis. Silakan instal secara manual."
    fi
else
    echo "Menggunakan GHC yang sudah terinstal"
fi

# Periksa CMake
if ! check_installed "cmake"; then
    echo "Menginstal CMake..."
    if command -v apt &> /dev/null; then
        sudo apt install -y cmake
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y cmake
    elif command -v pacman &> /dev/null; then
        sudo pacman -S cmake
    else
        echo "⚠ Tidak dapat menginstal CMake secara otomatis. Silakan instal secara manual."
    fi
else
    echo "Menggunakan CMake yang sudah terinstal"
fi

# Periksa Node.js
if ! check_installed "node"; then
    echo "Menginstal Node.js..."
    # Instal Node.js LTS menggunakan Nodesource
    curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
    sudo apt-get install -y nodejs
else
    echo "Menggunakan Node.js yang sudah terinstal"
fi

echo ""
echo "Instalasi dependensi selesai!"
echo ""
echo "Langkah selanjutnya:"
echo "1. cd /root/Fax/faxc"
echo "2. npm install"
echo "3. cd packages/lexer && cargo build --release && cd ../.."
echo "4. cd packages/parser && zig build && cd ../.."
echo "5. cd packages/sema && ghc -o bin/sema src/Main.hs && cd ../.."
echo "6. cd packages/optimizer && cargo build --release && cd ../.."
echo "7. cd packages/codegen/build && cmake .. && make && cd ../.."
echo "8. cd packages/runtime && zig build && cd ../.."