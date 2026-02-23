#!/bin/bash
# Fax Compiler - Code Formatting Script

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=========================================="
echo "Fax Compiler - Code Formatting"
echo "=========================================="
echo ""

cd "$PROJECT_DIR"

# Parse arguments
CHECK_ONLY=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --help, -h       Show this help message"
            echo "  --check, -c      Check formatting without modifying"
            echo "  --verbose, -v     Show detailed output"
            echo ""
            exit 0
            ;;
        --check|-c)
            CHECK_ONLY=true
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

# Check if rustfmt is installed
if ! command -v rustfmt &> /dev/null; then
    echo "rustfmt not found. Installing..."
    rustup component add rustfmt
fi

# Check if rustfmt is installed
if ! command -v rustfmt &> /dev/null; then
    echo "Error: rustfmt not available"
    exit 1
fi

if [ "$CHECK_ONLY" = true ]; then
    echo "Checking code formatting..."
    echo ""
    
    # Check formatting
    if cargo fmt --manifest-path "$PROJECT_DIR/Cargo.toml" -- --check; then
        echo ""
        echo "[OK] Code is properly formatted"
        exit 0
    else
        echo ""
        echo "[WARN] Code formatting issues found"
        echo "Run 'cargo fmt' to fix"
        exit 1
    fi
else
    echo "Formatting code..."
    cargo fmt --manifest-path "$PROJECT_DIR/Cargo.toml"
    
    echo ""
    echo "Formatting complete!"
fi
