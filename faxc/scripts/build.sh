#!/bin/bash
#
# build.sh - Build script for Fax Compiler
#
# Usage:
#   ./build.sh [OPTIONS] [CRATE_NAME]
#
# Options:
#   -r, --release           Build in release mode (optimized)
#   -d, --debug             Build in debug mode (default)
#   -w, --workspace         Build all workspace members
#   -f, --features <FEAT>   Build with specified features (comma-separated)
#   -h, --help              Show this help message
#
# Examples:
#   ./build.sh                          # Build default crate in debug mode
#   ./build.sh --release                # Build default crate in release mode
#   ./build.sh --workspace --release    # Build all crates in release mode
#   ./build.sh --features "parallel"    # Build with parallel feature enabled
#   ./build.sh faxc-lex                 # Build specific crate
#

set -euo pipefail

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default values
BUILD_MODE="debug"
BUILD_WORKSPACE=false
BUILD_FEATURES=""
BUILD_TARGET=""

# Colors for output (disabled if not a terminal)
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

# Print colored output
info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" >&2
}

error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

# Show help message
show_help() {
    sed -n '2,22p' "$0" | sed 's/^# \?//'
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -r|--release)
                BUILD_MODE="release"
                shift
                ;;
            -d|--debug)
                BUILD_MODE="debug"
                shift
                ;;
            -w|--workspace)
                BUILD_WORKSPACE=true
                shift
                ;;
            -f|--features)
                if [[ -z "${2:-}" ]]; then
                    error "Option --features requires an argument"
                    exit 1
                fi
                BUILD_FEATURES="$2"
                shift 2
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            -*)
                error "Unknown option: $1"
                echo "Use --help for usage information" >&2
                exit 1
                ;;
            *)
                # Positional argument: crate name
                BUILD_TARGET="$1"
                shift
                ;;
        esac
    done
}

# Build the project
build() {
    local cargo_args=()

    # Set build mode
    if [[ "$BUILD_MODE" == "release" ]]; then
        cargo_args+=("--release")
        info "Building in release mode..."
    else
        info "Building in debug mode..."
    fi

    # Set workspace or specific crate
    if [[ "$BUILD_WORKSPACE" == true ]]; then
        cargo_args+=("--workspace")
        info "Building all workspace members..."
    elif [[ -n "$BUILD_TARGET" ]]; then
        cargo_args+=("--package" "$BUILD_TARGET")
        info "Building crate: $BUILD_TARGET"
    fi

    # Add features if specified
    if [[ -n "$BUILD_FEATURES" ]]; then
        cargo_args+=("--features" "$BUILD_FEATURES")
        info "Using features: $BUILD_FEATURES"
    fi

    # Change to project root
    cd "$PROJECT_ROOT"

    # Run cargo build
    info "Running: cargo build ${cargo_args[*]}"
    if ! cargo build "${cargo_args[@]}"; then
        error "Build failed"
        exit 1
    fi

    # Print success message with binary location
    local profile="debug"
    if [[ "$BUILD_MODE" == "release" ]]; then
        profile="release"
    fi

    if [[ -n "$BUILD_TARGET" ]]; then
        success "Build complete for crate: $BUILD_TARGET"
        success "Binary: target/$profile/$BUILD_TARGET"
    elif [[ "$BUILD_WORKSPACE" == true ]]; then
        success "Build complete for all workspace members"
        success "Binaries: target/$profile/"
    else
        success "Build complete!"
        success "Binary: target/$profile/faxc"
    fi
}

# Main entry point
main() {
    parse_args "$@"
    build
}

main "$@"
