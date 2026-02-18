#!/bin/bash
#
# verify-msrv.sh - MSRV (Minimum Supported Rust Version) Verification Script
#
# Usage:
#   ./verify-msrv.sh [OPTIONS]
#
# Verifies that the project builds and tests pass on the MSRV (Rust 1.75).
#
# Options:
#   --msrv <VERSION>    MSRV version to test (default: 1.75)
#   --skip-tests        Skip running tests (only check/build)
#   --skip-clippy       Skip clippy checks
#   --workspace         Run on all workspace members (default)
#   -p, --package <NAME> Run on specific package only
#   -h, --help          Show this help message
#
# Examples:
#   ./verify-msrv.sh                      # Full MSRV verification
#   ./verify-msrv.sh --skip-tests         # Skip tests, only build
#   ./verify-msrv.sh --package faxc-lex   # Verify specific crate
#
# Exit codes:
#   0 - MSRV verification passed
#   1 - MSRV verification failed
#

set -euo pipefail

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default values
MSRV_VERSION="1.75"
RUN_TESTS=true
RUN_CLIPPY=true
CHECK_WORKSPACE=true
CHECK_PACKAGE=""

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
    sed -n '2,28p' "$0" | sed 's/^# \?//'
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --msrv)
                if [[ -z "${2:-}" ]]; then
                    error "Option --msrv requires an argument"
                    exit 1
                fi
                MSRV_VERSION="$2"
                shift 2
                ;;
            --skip-tests)
                RUN_TESTS=false
                shift
                ;;
            --skip-clippy)
                RUN_CLIPPY=false
                shift
                ;;
            --workspace)
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

# Check if rustup is available
check_rustup() {
    if ! command -v rustup &> /dev/null; then
        error "rustup is not installed. Please install rustup first:"
        error "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
}

# Check if MSRV toolchain is installed
check_msrv_toolchain() {
    info "Checking for Rust $MSRV_VERSION toolchain..."
    
    if ! rustup toolchain list | grep -q "^$MSRV_VERSION"; then
        info "Rust $MSRV_VERSION not found. Installing..."
        if ! rustup toolchain install "$MSRV_VERSION"; then
            error "Failed to install Rust $MSRV_VERSION"
            exit 1
        fi
    fi
    
    success "Rust $MSRV_VERSION toolchain is available"
}

# Verify MSRV is configured in Cargo.toml
check_msrv_config() {
    header "Checking MSRV configuration"
    
    local cargo_toml="$PROJECT_ROOT/Cargo.toml"
    
    if [[ ! -f "$cargo_toml" ]]; then
        error "Cargo.toml not found at $cargo_toml"
        exit 1
    fi
    
    local configured_msrv
    configured_msrv=$(grep -E "^rust-version\s*=" "$cargo_toml" | sed 's/.*"\([^"]*\)".*/\1/' || echo "")
    
    if [[ -z "$configured_msrv" ]]; then
        warn "No rust-version specified in Cargo.toml"
        warn "Consider adding: rust-version = \"$MSRV_VERSION\""
    elif [[ "$configured_msrv" != "$MSRV_VERSION" ]]; then
        warn "Configured rust-version ($configured_msrv) differs from MSRV being tested ($MSRV_VERSION)"
    else
        success "MSRV correctly configured as $MSRV_VERSION in Cargo.toml"
    fi
}

# Run cargo check with MSRV
run_check() {
    header "Running cargo check with Rust $MSRV_VERSION"
    
    local cargo_args=()
    
    if [[ "$CHECK_WORKSPACE" == true ]]; then
        cargo_args+=("--workspace")
        info "Checking all workspace members..."
    elif [[ -n "$CHECK_PACKAGE" ]]; then
        cargo_args+=("--package" "$CHECK_PACKAGE")
        info "Checking package: $CHECK_PACKAGE"
    fi
    
    cd "$PROJECT_ROOT"
    
    if rustup run "$MSRV_VERSION" cargo check "${cargo_args[@]}"; then
        success "cargo check passed on Rust $MSRV_VERSION"
        return 0
    else
        error "cargo check failed on Rust $MSRV_VERSION"
        error "This may indicate use of features not available in $MSRV_VERSION"
        FAILED_CHECKS+=("cargo check")
        return 1
    fi
}

# Run cargo build with MSRV
run_build() {
    header "Running cargo build with Rust $MSRV_VERSION"
    
    local cargo_args=()
    
    if [[ "$CHECK_WORKSPACE" == true ]]; then
        cargo_args+=("--workspace")
        info "Building all workspace members..."
    elif [[ -n "$CHECK_PACKAGE" ]]; then
        cargo_args+=("--package" "$CHECK_PACKAGE")
        info "Building package: $CHECK_PACKAGE"
    fi
    
    cd "$PROJECT_ROOT"
    
    if rustup run "$MSRV_VERSION" cargo build "${cargo_args[@]}"; then
        success "cargo build passed on Rust $MSRV_VERSION"
        return 0
    else
        error "cargo build failed on Rust $MSRV_VERSION"
        FAILED_CHECKS+=("cargo build")
        return 1
    fi
}

# Run cargo test with MSRV
run_tests() {
    header "Running cargo test with Rust $MSRV_VERSION"
    
    local cargo_args=()
    
    if [[ "$CHECK_WORKSPACE" == true ]]; then
        cargo_args+=("--workspace")
        info "Testing all workspace members..."
    elif [[ -n "$CHECK_PACKAGE" ]]; then
        cargo_args+=("--package" "$CHECK_PACKAGE")
        info "Testing package: $CHECK_PACKAGE"
    fi
    
    cd "$PROJECT_ROOT"
    
    if rustup run "$MSRV_VERSION" cargo test "${cargo_args[@]}"; then
        success "cargo test passed on Rust $MSRV_VERSION"
        return 0
    else
        error "cargo test failed on Rust $MSRV_VERSION"
        FAILED_CHECKS+=("cargo test")
        return 1
    fi
}

# Run cargo clippy with MSRV
run_clippy() {
    header "Running cargo clippy with Rust $MSRV_VERSION"
    
    # Check if clippy component is available for MSRV
    if ! rustup run "$MSRV_VERSION" rustc --print sysroot >/dev/null 2>&1; then
        warn "Clippy may not be available for Rust $MSRV_VERSION, skipping..."
        return 0
    fi
    
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
    
    cd "$PROJECT_ROOT"
    
    if rustup run "$MSRV_VERSION" cargo clippy "${cargo_args[@]}"; then
        success "cargo clippy passed on Rust $MSRV_VERSION"
        return 0
    else
        error "cargo clippy failed on Rust $MSRV_VERSION"
        FAILED_CHECKS+=("cargo clippy")
        return 1
    fi
}

# Print summary
print_summary() {
    echo ""
    if [[ ${#FAILED_CHECKS[@]} -eq 0 ]]; then
        success "═══════════════════════════════════════════════════════"
        success "  MSRV Verification PASSED for Rust $MSRV_VERSION"
        success "  All checks completed successfully!"
        success "═══════════════════════════════════════════════════════"
        return 0
    else
        error "═══════════════════════════════════════════════════════"
        error "  MSRV Verification FAILED for Rust $MSRV_VERSION"
        error "  The following checks failed:"
        for check in "${FAILED_CHECKS[@]}"; do
            error "    - $check"
        done
        error ""
        error "  Your code may be using features newer than Rust $MSRV_VERSION"
        error "  or dependencies may require a newer Rust version."
        error "═══════════════════════════════════════════════════════"
        return 1
    fi
}

# Main entry point
main() {
    parse_args "$@"
    
    header "MSRV Verification Script"
    info "Testing against Rust $MSRV_VERSION"
    info "Project root: $PROJECT_ROOT"
    
    check_rustup
    check_msrv_toolchain
    check_msrv_config
    
    local exit_code=0
    
    # Run checks in order of speed (fastest first)
    run_check || exit_code=1
    run_build || exit_code=1
    
    if [[ "$RUN_TESTS" == true ]]; then
        run_tests || exit_code=1
    fi
    
    if [[ "$RUN_CLIPPY" == true ]]; then
        run_clippy || exit_code=1
    fi
    
    print_summary
    exit $exit_code
}

main "$@"
