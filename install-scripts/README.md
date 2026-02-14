# Fax Installation Scripts

This directory contains the official installation scripts for Fax.

## Quick Install

```bash
curl --proto '=https' --tlsv1.2 -sSf https://luvion1.github.io/Fax/install.sh | sh
```

## What It Does

1. **Dependency Check**: Verifies all required tools are installed
2. **Platform Detection**: Automatically detects your OS and architecture
3. **Installation**: Clones Fax to `~/.fax/repo`
4. **Build**: Compiles all compiler components
5. **PATH Setup**: Adds `~/.fax/bin` to your shell configuration

## Features

- ✅ Secure HTTPS download
- ✅ Cross-platform (Linux, macOS)
- ✅ Multi-architecture (x86_64, aarch64)
- ✅ Dependency validation
- ✅ Automatic PATH configuration
- ✅ Clean uninstall support

## Files

- `install.sh` - Main installation script
- `uninstall.sh` - Uninstallation script

## Security

The installer uses HTTPS with TLS 1.2 for secure downloads. It does not require sudo for installation (installs to `~/.fax`).

## Manual Installation

See the [Installation Guide](https://luvion1.github.io/Fax/getting-started/installation/) for manual installation instructions.
