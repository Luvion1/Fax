#!/bin/bash
# Fax Compiler - Clean Build Artifacts Script

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=========================================="
echo "Fax Compiler - Clean Build Artifacts"
echo "=========================================="
echo ""

# Parse arguments
CLEAN_TARGET=""
CLEAN_DEPS=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --help, -h       Show this help message"
            echo "  --target, -t      Clean specific target (debug/release/all)"
            echo "  --deps, -d       Also clean dependencies cache"
            echo "  --verbose, -v     Show detailed output"
            echo ""
            echo "Examples:"
            echo "  $0                  # Clean all build artifacts"
            echo "  $0 --target release # Clean only release build"
            echo "  $0 --deps          # Clean with cargo cache"
            exit 0
            ;;
        --target|-t)
            CLEAN_TARGET="$2"
            shift 2
            ;;
        --deps|-d)
            CLEAN_DEPS=true
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

cd "$PROJECT_DIR"

# Clean target directory
if [ -d "target" ]; then
    if [ -z "$CLEAN_TARGET" ] || [ "$CLEAN_TARGET" == "all" ]; then
        echo "Cleaning all build artifacts..."
        rm -rf target
    elif [ "$CLEAN_TARGET" == "debug" ]; then
        echo "Cleaning debug build..."
        rm -rf target/debug
    elif [ "$CLEAN_TARGET" == "release" ]; then
        echo "Cleaning release build..."
        rm -rf target/release
    fi
fi

# Clean Cargo build cache
if [ "$CLEAN_DEPS" = true ]; then
    echo "Cleaning Cargo cache..."
    rm -rf ~/.cargo/registry/cache
    rm -rf ~/.cargo/registry/index
    rm -rf ~/.cargo/git/db
    rm -rf ~/.cargo/.package-cache
    
    # Clean cargo target directory if in workspace
    cargo clean --manifest-path "$PROJECT_DIR/Cargo.toml"
fi

# Also clean any .divo files or temp files
echo "Cleaning temporary files..."
find "$PROJECT_DIR" -name "*.tmp" -delete 2>/dev/null || true
find "$PROJECT_DIR" -name "*.bak" -delete 2>/dev/null || true
find "$PROJECT_DIR" -name "*~" -delete 2>/dev/null || true

# Clean .divo if exists
if [ -d ".divo" ]; then
    rm -rf .divo
fi

echo ""
echo "Clean complete!"
