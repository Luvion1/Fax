---
title: Installation
description: Install Fax programming language on your system
---

# Installation

Fax can be installed using our official installer script or manually from source.

## Quick Install (Recommended)

The easiest way to install Fax is using our installer script:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://luvion1.github.io/Fax/install.sh | sh
```

This will:
1. Check that all dependencies are installed
2. Clone the Fax repository to `~/.fax/repo`
3. Install Node.js dependencies
4. Build all compiler components
5. Add `~/.fax/bin` to your PATH

After installation, restart your terminal or run:

```bash
source ~/.bashrc  # or ~/.zshrc for zsh users
```

Verify the installation:

```bash
faxt --version
```

## Manual Installation

If you prefer to install manually:

### 1. Clone the Repository

```bash
git clone https://github.com/Luvion1/Fax.git
cd Fax
```

### 2. Install Dependencies

Fax requires several tools to be installed on your system:

#### macOS (using Homebrew)

```bash
brew install git python3 node rust zig ghc
```

#### Ubuntu/Debian

```bash
# Install basics
sudo apt-get update
sudo apt-get install -y git python3 python3-pip

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Zig
sudo snap install zig --classic

# Install GHC (Haskell)
sudo apt-get install -y ghc
```

### 3. Build Fax

```bash
# Install Node.js dependencies
npm install

# Build all compiler components
make build
```

### 4. Add to PATH

Add the following to your shell configuration file (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
export PATH="/path/to/Fax:$PATH"
```

Or create symlinks:

```bash
ln -sf $(pwd)/faxt/main.py /usr/local/bin/faxt
```

## Verify Installation

Check that everything is working:

```bash
# Check all tools are available
faxt doctor

# Run a test program
echo 'fn main() { print("Hello, Fax!"); }' > hello.fax
faxt run hello.fax
```

## Platform Support

Fax officially supports:

| Platform | Architecture | Status |
|----------|-------------|--------|
| Linux | x86_64 | ✅ Fully supported |
| Linux | aarch64 | ✅ Fully supported |
| macOS | x86_64 | ✅ Fully supported |
| macOS | aarch64 (Apple Silicon) | ✅ Fully supported |
| Windows | x86_64 | 🚧 Coming soon |

## Update Fax

To update to the latest version:

### If installed via installer script:

```bash
cd ~/.fax/repo
git pull
npm install
make build
```

### If installed manually:

```bash
cd /path/to/Fax
git pull
make clean
make build
```

## Uninstall

To remove Fax from your system:

### If installed via installer script:

```bash
~/.fax/bin/fax-uninstall
```

Or manually:

```bash
rm -rf ~/.fax
# Also remove the PATH export from your shell config
```

### If installed manually:

Simply delete the Fax directory:

```bash
rm -rf /path/to/Fax
# Remove any symlinks you created
```

## Troubleshooting

### "Command not found" after installation

Make sure you've restarted your terminal or sourced your shell configuration:

```bash
source ~/.bashrc  # or ~/.zshrc
```

### Missing dependencies

If you see errors about missing tools, install them following the [Manual Installation](#manual-installation) section above.

### Build fails

Try cleaning and rebuilding:

```bash
make clean
make build
```

For more help, see the [Troubleshooting Guide](/Fax/guides/troubleshooting/).

## Next Steps

Once installed, proceed to the [Quick Start](/Fax/getting-started/quick-start/) guide to write your first Fax program!
