#!/bin/bash
# Fax Compiler - Development Setup Script
# Sets up complete development environment

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=========================================="
echo "Fax Compiler - Development Setup"
echo "=========================================="
echo ""

cd "$PROJECT_DIR"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print colored message
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

# Check if running in CI
CI_MODE=false
if [ -n "$CI" ]; then
    CI_MODE=true
fi

# Parse arguments
SKIP_DEPS=false
FORCE_BUILD=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --help, -h       Show this help message"
            echo "  --skip-deps      Skip dependency installation"
            echo "  --force          Force rebuild even if binary exists"
            echo "  --verbose, -v    Show detailed output"
            echo ""
            exit 0
            ;;
        --skip-deps)
            SKIP_DEPS=true
            shift
            ;;
        --force|-f)
            FORCE_BUILD=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "Step 1: Checking Rust installation..."
echo ""

# Check Rust
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    print_status "Rust installed: $RUST_VERSION"
else
    print_error "Rust not found!"
    echo ""
    echo "Please install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version | awk '{print $2}')
    print_status "Cargo installed: $CARGO_VERSION"
fi

# Check if using nightly (required for some features)
if rustup show active-toolchain 2>/dev/null | grep -q nightly; then
    print_status "Using nightly Rust"
fi

echo ""
echo "Step 2: Checking LLVM installation..."
echo ""

# Check LLVM
LLVM_FOUND=false

if [ -n "$LLVM_SYS_20_PREFIX" ]; then
    if [ -f "$LLVM_SYS_20_PREFIX/bin/llvm-config" ]; then
        LLVM_VERSION=$($LLVM_SYS_20_PREFIX/bin/llvm-config --version 2>/dev/null)
        print_status "LLVM found at LLVM_SYS_20_PREFIX: $LLVM_VERSION"
        LLVM_FOUND=true
    fi
fi

if [ "$LLVM_FOUND" = false ] && command -v llvm-config-20 &> /dev/null; then
    LLVM_VERSION=$(llvm-config-20 --version 2>/dev/null)
    print_status "LLVM 20 found: $LLVM_VERSION"
    export LLVM_SYS_20_PREFIX=$(dirname $(dirname $(which llvm-config-20)))
    LLVM_FOUND=true
fi

if [ "$LLVM_FOUND" = false ] && command -v llvm-config &> /dev/null; then
    LLVM_VERSION=$(llvm-config --version 2>/dev/null)
    print_warning "LLVM found but version not 20: $LLVM_VERSION"
    echo "Fax requires LLVM 20. Run ./scripts/install.sh to install."
fi

if [ "$LLVM_FOUND" = false ]; then
    print_error "LLVM 20 not found!"
    echo ""
    echo "Please install LLVM 20. Run:"
    echo "  ./scripts/install.sh"
    echo ""
    echo "Or manually install and set:"
    echo "  export LLVM_SYS_20_PREFIX=/path/to/llvm20"
    exit 1
fi

echo ""
echo "Step 3: Installing development tools..."
echo ""

# Install rustfmt if not present
if ! command -v rustfmt &> /dev/null; then
    print_status "Installing rustfmt..."
    rustup component add rustfmt
else
    print_status "rustfmt already installed"
fi

# Install clippy if not present
if ! rustup component list | grep -q clippy 2>/dev/null; then
    print_status "Installing clippy..."
    rustup component add clippy
else
    print_status "clippy already installed"
fi

# Install miri if not present (optional)
if ! rustup component list | grep -q miri 2>/dev/null; then
    if [ "$VERBOSE" = true ]; then
        print_status "Installing miri (optional)..."
        rustup component add miri 2>/dev/null || true
    fi
fi

echo ""
echo "Step 4: Building Fax compiler..."
echo ""

# Check if binary already exists
FAXC_BIN="$PROJECT_DIR/target/debug/faxc-drv"

if [ -f "$FAXC_BIN" ] && [ "$FORCE_BUILD" = false ]; then
    print_status "Fax compiler already built: $FAXC_BIN"
else
    if [ "$CI_MODE" = true ]; then
        # In CI, build with more optimizations
        cargo build -p faxc-drv --release
        FAXC_BIN="$PROJECT_DIR/target/release/faxc-drv"
    else
        cargo build -p faxc-drv
    fi
    
    if [ $? -eq 0 ]; then
        print_status "Fax compiler built successfully!"
    else
        print_error "Failed to build Fax compiler"
        exit 1
    fi
fi

echo ""
echo "Step 5: Verifying installation..."
echo ""

# Verify compiler works
if [ -f "$FAXC_BIN" ]; then
    if "$FAXC_BIN" --version &> /dev/null; then
        print_status "Fax compiler is working!"
        echo ""
        echo "Version: $($FAXC_BIN --version)"
    else
        print_warning "Fax compiler exists but --version failed"
    fi
fi

echo ""
echo "=========================================="
echo "Setup Complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "  1. Run tests:     ./scripts/test.sh"
echo "  2. Build release: ./scripts/build.sh --release"
echo "  3. Run examples:  ./scripts/run.sh --list"
echo ""
echo "Available scripts:"
echo "  ./scripts/build.sh   - Build compiler"
echo "  ./scripts/test.sh   - Run tests"
echo "  ./scripts/run.sh    - Run examples"
echo "  ./scripts/clean.sh - Clean build artifacts"
echo "  ./scripts/fmt.sh    - Format code"
echo "  ./scripts/clippy.sh - Run linter"
echo "  ./scripts/install.sh - Install dependencies"
echo ""
