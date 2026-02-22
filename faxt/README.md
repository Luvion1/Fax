# Faxt

<p align="center">
  <img src="../faxc/docs/rubah-arktik.svg" width="80" alt="Fax Arctic Fox">
</p>

A CLI tool for fax operations, written in Rust.

[![Build Status](https://img.shields.io/github/actions/workflow/status/Fax/faxt/ci.yml?branch=main)](https://github.com/Fax/faxt/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://rustup.rs)

## Overview

Faxt is a command-line utility designed to streamline fax-related operations. It provides a clean, intuitive interface for:

- **Initializing** new fax projects with proper directory structure
- **Building** artifacts from input files
- **Converting** files between different formats

## Features

- 🚀 Fast and reliable, built with Rust
- 📦 Cross-platform support (Linux, macOS, Windows)
- 🔧 Configurable via CLI arguments or configuration files
- 📝 Comprehensive error messages and help documentation
- 🧪 Well-tested with unit and integration tests

## Installation

### Prerequisites

- Rust 1.75 or later ([install via rustup](https://rustup.rs))

### From Source

```bash
# Clone the repository
git clone https://github.com/Fax/faxt.git
cd faxt

# Build in release mode
cargo build --release

# Install the binary
cargo install --path .
```

### Using Cargo

```bash
cargo install faxt
```

### Pre-built Binaries

Download pre-built binaries from the [releases page](https://github.com/Fax/faxt/releases).

## Usage

### Getting Help

```bash
# General help
faxt --help

# Help for a specific command
faxt init --help
faxt build --help
faxt convert --help

# Version information
faxt --version
```

### Commands

#### `init` - Initialize a new project

Creates a new faxt project with the standard directory structure and configuration files.

```bash
# Initialize in current directory
faxt init

# Initialize with a specific name
faxt init --name my-project

# Initialize in a specific directory
faxt init --path /path/to/project

# Force initialization even if directory is not empty
faxt init --force
```

#### `build` - Build project artifacts

Processes input files and generates output artifacts.

```bash
# Build with default settings
faxt build

# Specify input and output directories
faxt build --input ./src --output ./dist

# Clean before building
faxt build --clean

# Disable optimizations
faxt build --no-optimize

# Set number of parallel jobs
faxt build --jobs 4

# Specify target architecture
faxt build --target x86_64-unknown-linux-gnu
```

#### `convert` - Convert files between formats

Converts input files to different output formats.

```bash
# Convert a single file
faxt convert input.txt

# Convert with specific format
faxt convert input.txt --format pdf

# Convert with quality setting
faxt convert input.txt --format jpeg --quality 85

# Convert multiple files
faxt convert file1.txt file2.txt file3.txt

# Specify output directory
faxt convert input.txt --output ./output

# Overwrite existing files
faxt convert input.txt --force

# Convert recursively
faxt convert ./input --recursive --format pdf
```

### Global Options

| Option | Description | Environment Variable |
|--------|-------------|---------------------|
| `-v, --verbose` | Enable verbose output | `FAXT_VERBOSE` |
| `-c, --config <PATH>` | Path to configuration file | `FAXT_CONFIG` |
| `--no-color` | Disable colored output | `FAXT_NO_COLOR` |
| `-h, --help` | Print help information | - |
| `-V, --version` | Print version information | - |

## Configuration

Faxt can be configured via a TOML configuration file. The configuration is loaded from the following locations (in order of precedence):

1. Path specified via `--config` or `FAXT_CONFIG`
2. `./faxt.toml` (current directory)
3. `~/.config/faxt/faxt.toml` (user config)
4. Default configuration

### Configuration File Format

```toml
# Global settings
verbose = false
output_dir = "output"
input_dir = "input"

# Build settings
[build]
optimize = true
target = "x86_64-unknown-linux-gnu"
jobs = 4

# Convert settings
[convert]
format = "pdf"
quality = 90
preserve_metadata = true
```

### Default Configuration

| Setting | Default Value |
|---------|---------------|
| `verbose` | `false` |
| `output_dir` | `"output"` |
| `input_dir` | `"input"` |
| `build.optimize` | `true` |
| `build.jobs` | Number of CPU cores |
| `convert.format` | `"pdf"` |
| `convert.quality` | `90` |
| `convert.preserve_metadata` | `true` |

## Project Structure

When you run `faxt init`, the following structure is created:

```
project/
├── faxt.toml      # Configuration file
├── input/         # Input files directory
├── output/        # Output files directory
├── build/         # Build artifacts directory
└── .faxt/         # Internal faxt data
```

## Development

### Setting Up the Development Environment

```bash
# Clone the repository
git clone https://github.com/Fax/faxt.git
cd faxt

# Ensure you have the correct Rust toolchain
rustup show

# Install development dependencies
cargo install cargo-watch
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_init_command

# Run tests with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Check for errors without building
cargo check
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy for linting
cargo clippy -- -D warnings

# Run all checks
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

### Running the CLI During Development

```bash
# Run with arguments
cargo run -- init --name test-project

# Run with verbose output
cargo run -- --verbose build

# Run in release mode
cargo run --release -- convert input.txt --format pdf
```

## Architecture

```
src/
├── main.rs          # CLI entry point and argument parsing
├── error.rs         # Error types and handling
├── config.rs        # Configuration loading and management
└── commands/        # Command implementations
    ├── mod.rs       # Command module exports
    ├── init.rs      # Project initialization
    ├── build.rs     # Build command
    └── convert.rs   # File conversion
```

### Error Handling

Faxt uses a custom error type based on `thiserror` for structured error handling:

- `FaxtError::Config` - Configuration-related errors
- `FaxtError::FileOperation` - File I/O errors
- `FaxtError::Validation` - Input validation errors
- `FaxtError::CommandExecution` - Command execution failures
- `FaxtError::Io` - Standard IO errors
- `FaxtError::Json` - JSON serialization errors

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Quick Start for Contributors

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linters (`cargo test && cargo clippy`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [clap](https://github.com/clap-rs/clap) for argument parsing
- Error handling powered by [thiserror](https://github.com/dtolnay/thiserror) and [anyhow](https://github.com/dtolnay/anyhow)
- Logging via [tracing](https://github.com/tokio-rs/tracing)

## Support

- **Issues**: [GitHub Issues](https://github.com/Fax/faxt/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Fax/faxt/discussions)
