---
title: Installation
description: Install Fax programming language
---

## Prerequisites

Before installing Fax, ensure you have the following tools installed:

### Required Tools

| Tool | Version | Purpose |
|------|---------|---------|
| **Rust** | Latest stable | Lexer, Optimizer |
| **Zig** | 0.13+ | Parser, Runtime |
| **GHC** | 9.0+ | Semantic Analyzer |
| **C++ Compiler** | GCC/Clang | Code Generator |
| **Node.js** | 18+ | Tooling |
| **Python** | 3.8+ | CLI |

### Install Prerequisites

#### macOS (using Homebrew)

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Zig
brew install zig

# GHC (Haskell)
brew install ghc

# Node.js
brew install node
```

#### Ubuntu/Debian

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Zig
sudo snap install zig --classic

# GHC
sudo apt-get install ghc

# Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```

## Install Fax

### From Source

```bash
# Clone the repository
git clone https://github.com/Luvion1/Fax.git
cd Fax

# Install Node.js dependencies
npm install

# Build all compiler components
make build

# Verify installation
python3 faxt/main.py doctor
```

### Verify Installation

```bash
# Check all tools are available
python3 faxt/main.py doctor

# Should output something like:
# ✓ Rust installed
# ✓ Zig installed
# ✓ GHC installed
# ✓ Node.js installed
```

## Update

To update Fax to the latest version:

```bash
git pull origin main
make clean
make build
```

## Uninstall

Simply remove the Fax directory:

```bash
cd ..
rm -rf Fax
```
