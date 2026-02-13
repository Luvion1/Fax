---
sidebar_position: 1
---

# Installation

## Prerequisites

Install the following tools based on your operating system:

### Required Tools

| Tool | Version | Purpose |
|------|---------|---------|
| **Rust** | 1.93.0 | Lexer, Optimizer |
| **Zig** | 0.14.1 | Parser, Runtime |
| **GHC** | 9.6.6 | Semantic Analyzer |
| **GCC** | 15.2.0 | Code Generator |
| **Node.js** | 20.20.0 | CLI Tooling |
| **npm** | 10.8.2 | Package Manager |

### Installing on Linux

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Zig
wget https://ziglang.org/download/0.14.1/zig-linux-x86_64-0.14.1.tar.xz
tar -xf zig-linux-x86_64-0.14.1.tar.xz
sudo mv zig-linux-x86_64-0.14.1 /opt/zig
export PATH=$PATH:/opt/zig

# GHC (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install ghc cabal-install

# Node.js (via nvm)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20

# GCC
sudo apt-get install build-essential
```

### Installing on macOS

```bash
# Homebrew
brew install rust zig ghc node

# Or via nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
```

### Installing on Windows

```bash
# Via Chocolatey
choco install rust zig nodejs

# Or manually download from:
# - https://rustup.rs
# - https://ziglang.org/download
# - https://nodejs.org
```

## Verify Installation

```bash
# Check all tools
rustc --version    # rustc 1.93.0
cargo --version   # cargo 1.93.0
zig version       # 0.14.1
ghc --version     # 9.6.6
node --version    # v20.20.0
npm --version     # 10.8.2
```

## Clone and Setup

```bash
# Clone the repository
git clone https://github.com/Luvion1/Fax.git
cd Fax

# Install dependencies
npm install

# Build all components
make build
```

## Next Steps

- [Quick Start Guide](/docs/getting-started/quick-start)
- [Examples](/docs/getting-started/examples)
