#!/bin/bash
# Fax Compiler - Clippy Linting Script

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=========================================="
echo "Fax Compiler - Clippy Linting"
echo "=========================================="
echo ""

cd "$PROJECT_DIR"

# Parse arguments
FIX_MODE=false
VERBOSE=false
TARGET=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --help, -h       Show this help message"
            echo "  --fix, -f        Try to fix warnings automatically"
            echo "  --target TARGET   Check specific target (debug/release)"
            echo "  --verbose, -v     Show detailed output"
            echo ""
            exit 0
            ;;
        --fix|-f)
            FIX_MODE=true
            shift
            ;;
        --target|-t)
            TARGET="$2"
            shift 2
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

# Check if clippy is installed
if ! rustup component list | grep -q clippy 2>/dev/null; then
    echo "Clippy not found. Installing..."
    rustup component add clippy
fi

# Build args
BUILD_ARGS="--workspace"

if [ -n "$TARGET" ]; then
    if [ "$TARGET" == "release" ]; then
        BUILD_ARGS="$BUILD_ARGS --release"
    fi
fi

if [ "$FIX_MODE" = true ]; then
    echo "Running clippy with fix..."
    cargo clippy $BUILD_ARGS -- -D warnings -A clippy::all -W clippy::style -W clippy::perf -W clippy::pedantic
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "Clippy check passed with fixes!"
    fi
else
    echo "Running clippy check..."
    cargo clippy $BUILD_ARGS -- -D warnings -A clippy::all -W clippy::style -W clippy::perf
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "Clippy check passed!"
    else
        echo ""
        echo "Clippy found issues. Run with --fix to attempt automatic fixes"
    fi
fi
