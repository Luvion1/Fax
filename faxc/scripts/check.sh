#!/bin/bash
#
# check.sh - Pre-commit check script for Fax Compiler
#
# Usage:
#   ./check.sh [OPTIONS]
#
# Runs a series of fast checks suitable for pre-commit validation:
#   1. cargo check        - Fast compilation check
#   2. cargo clippy       - Linting with clippy
#   3. cargo fmt --check  - Code formatting verification
#
# Options:
#   -w, --workspace         Check all workspace members (default)
#   -p, --package <NAME>    Check specific package only
#   --no-clippy             Skip clippy checks
#   --no-fmt                Skip fmt checks
#   --fix                   Auto-fix clippy warnings (when possible)
#   --allow-dirty           Allow uncommitted changes when fixing
#   -h, --help              Show this help message
#
# Examples:
#   ./check.sh                      # Run all checks on workspace
#   ./check.sh --package faxc-lex   # Check specific crate
#   ./check.sh --no-fmt             # Skip formatting check
#   ./check.sh --fix                # Auto-fix clippy warnings
#
# Exit codes:
#   0 - All checks passed
#   1 - One or more checks failed
#

set -euo pipefail

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default values
CHECK_WORKSPACE=true
CHECK_PACKAGE=""
RUN_CLIPPY=true
RUN_FMT=true
CLIPPY_FIX=false
ALLOW_DIRTY=false

# Track failures
FAILED_CHECKS=()

# Colors for output (disabled if not a terminal)
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    CYAN='\033[0;36m'
    NC='\033[0m' # No Color
    BOLD='\033[1m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    CYAN=''
    NC=''
    BOLD=''
fi

# Print colored output
info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

success() {
    echo -e "${GREEN}[PASS]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" >&2
}

error() {
    echo -e "${RED}[FAIL]${NC} $*" >&2
}

header() {
    echo ""
    echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}${CYAN}  $*${NC}"
    echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════${NC}"
}

# Show help message
show_help() {
    sed -n '2,32p' "$0" | sed 's/^# \?//'
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -w|--workspace)
                CHECK_WORKSPACE=true
                shift
                ;;
            -p|--package)
                if [[ -z "${2:-}" ]]; then
                    error "Option --package requires an argument"
                    exit 1
                fi
                CHECK_PACKAGE="$2"
                CHECK_WORKSPACE=false
                shift 2
                ;;
            --no-clippy)
                RUN_CLIPPY=false
                shift
                ;;
            --no-fmt)
                RUN_FMT=false
                shift
                ;;
            --fix)
                CLIPPY_FIX=true
                shift
                ;;
            --allow-dirty)
                ALLOW_DIRTY=true
                shift
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
                error "Unexpected argument: $1"
                exit 1
                ;;
        esac
    done
}

# Check for uncommitted changes when using --fix
check_dirty() {
    if [[ "$CLIPPY_FIX" == true && "$ALLOW_DIRTY" == false ]]; then
        cd "$PROJECT_ROOT"
        if [[ -n "$(git status --porcelain 2>/dev/null || true)" ]]; then
            error "Working directory has uncommitted changes"
            error "Commit your changes first, or use --allow-dirty"
            exit 1
        fi
    fi
}

# Run cargo check
run_check() {
    header "Running cargo check"

    local cargo_args=()

    if [[ "$CHECK_WORKSPACE" == true ]]; then
        cargo_args+=("--workspace")
        info "Checking all workspace members..."
    elif [[ -n "$CHECK_PACKAGE" ]]; then
        cargo_args+=("--package" "$CHECK_PACKAGE")
        info "Checking package: $CHECK_PACKAGE"
    fi

    cd "$PROJECT_ROOT"

    if cargo check "${cargo_args[@]}"; then
        success "cargo check passed"
        return 0
    else
        error "cargo check failed"
        FAILED_CHECKS+=("cargo check")
        return 1
    fi
}

# Run cargo clippy
run_clippy() {
    header "Running cargo clippy"

    local cargo_args=()

    if [[ "$CHECK_WORKSPACE" == true ]]; then
        cargo_args+=("--workspace")
        info "Linting all workspace members..."
    elif [[ -n "$CHECK_PACKAGE" ]]; then
        cargo_args+=("--package" "$CHECK_PACKAGE")
        info "Linting package: $CHECK_PACKAGE"
    fi

    # Deny warnings for stricter checking
    cargo_args+=("--" "-D" "warnings")

    if [[ "$CLIPPY_FIX" == true ]]; then
        info "Auto-fixing clippy warnings..."
        cargo_args=("--fix" "${cargo_args[@]}")
        if [[ "$ALLOW_DIRTY" == true ]]; then
            cargo_args+=("--allow-dirty")
        fi
    fi

    cd "$PROJECT_ROOT"

    if cargo clippy "${cargo_args[@]}"; then
        success "cargo clippy passed"
        return 0
    else
        error "cargo clippy failed"
        FAILED_CHECKS+=("cargo clippy")
        return 1
    fi
}

# Run cargo fmt --check
run_fmt() {
    header "Running cargo fmt --check"

    local cargo_args=()

    if [[ "$CHECK_WORKSPACE" == true ]]; then
        cargo_args+=("--check" "--workspace")
        info "Checking formatting for all workspace members..."
    elif [[ -n "$CHECK_PACKAGE" ]]; then
        cargo_args+=("--check" "--package" "$CHECK_PACKAGE")
        info "Checking formatting for package: $CHECK_PACKAGE"
    else
        cargo_args+=("--check")
    fi

    cd "$PROJECT_ROOT"

    if cargo fmt "${cargo_args[@]}"; then
        success "cargo fmt check passed"
        return 0
    else
        error "cargo fmt check failed - code is not properly formatted"
        error "Run 'cargo fmt' to fix formatting issues"
        FAILED_CHECKS+=("cargo fmt")
        return 1
    fi
}

# Print summary
print_summary() {
    echo ""
    if [[ ${#FAILED_CHECKS[@]} -eq 0 ]]; then
        success "════════════════════════════════════════"
        success "  All checks passed! Ready to commit."
        success "════════════════════════════════════════"
        return 0
    else
        error "════════════════════════════════════════"
        error "  The following checks failed:"
        for check in "${FAILED_CHECKS[@]}"; do
            error "    - $check"
        done
        error "════════════════════════════════════════"
        return 1
    fi
}

# Main entry point
main() {
    parse_args "$@"

    check_dirty

    local exit_code=0

    # Run checks in order of speed (fastest first)
    run_check || exit_code=1

    if [[ "$RUN_CLIPPY" == true ]]; then
        run_clippy || exit_code=1
    fi

    if [[ "$RUN_FMT" == true ]]; then
        run_fmt || exit_code=1
    fi

    print_summary
    exit $exit_code
}

main "$@"
