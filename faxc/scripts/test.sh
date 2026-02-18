#!/bin/bash
#
# test.sh - Test script for Fax Compiler
#
# Usage:
#   ./test.sh [OPTIONS] [TEST_FILTER]
#
# Options:
#   -r, --release           Run tests in release mode (default for speed)
#   -d, --debug             Run tests in debug mode
#   -w, --workspace         Run tests for all workspace members (default)
#   -c, --coverage          Collect code coverage (requires cargo-tarpaulin or grcov)
#   -f, --features <FEAT>   Run tests with specified features
#   -t, --test <NAME>       Run only tests matching NAME (filter)
#   --nocapture             Show stdout/stderr from tests
#   --exact                 Match test names exactly
#   -h, --help              Show this help message
#
# Examples:
#   ./test.sh                           # Run all tests in release mode
#   ./test.sh --debug                   # Run all tests in debug mode
#   ./test.sh --coverage                # Run tests with coverage report
#   ./test.sh test_lex                  # Run only tests matching "test_lex"
#   ./test.sh --exact my_test           # Run test named exactly "my_test"
#   ./test.sh --nocapture               # Show test output
#

set -euo pipefail

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default values
TEST_MODE="release"  # Default to release for speed
TEST_WORKSPACE=true
TEST_COVERAGE=false
TEST_FEATURES=""
TEST_FILTER=""
TEST_NOCAPTURE=false
TEST_EXACT=false

# Colors for output (disabled if not a terminal)
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    CYAN='\033[0;36m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    CYAN=''
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
    sed -n '2,27p' "$0" | sed 's/^# \?//'
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -r|--release)
                TEST_MODE="release"
                shift
                ;;
            -d|--debug)
                TEST_MODE="debug"
                shift
                ;;
            -w|--workspace)
                TEST_WORKSPACE=true
                shift
                ;;
            -c|--coverage)
                TEST_COVERAGE=true
                shift
                ;;
            -f|--features)
                if [[ -z "${2:-}" ]]; then
                    error "Option --features requires an argument"
                    exit 1
                fi
                TEST_FEATURES="$2"
                shift 2
                ;;
            -t|--test)
                if [[ -z "${2:-}" ]]; then
                    error "Option --test requires an argument"
                    exit 1
                fi
                TEST_FILTER="$2"
                shift 2
                ;;
            --nocapture)
                TEST_NOCAPTURE=true
                shift
                ;;
            --exact)
                TEST_EXACT=true
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
                # Positional argument: test filter
                if [[ -n "$TEST_FILTER" ]]; then
                    error "Multiple test filters provided: $TEST_FILTER and $1"
                    exit 1
                fi
                TEST_FILTER="$1"
                shift
                ;;
        esac
    done
}

# Run tests with coverage
run_coverage() {
    info "Running tests with coverage..."

    # Check for cargo-tarpaulin
    if command -v cargo-tarpaulin &> /dev/null; then
        info "Using cargo-tarpaulin for coverage..."
        local tarpaulin_args=("--workspace" "--out" "Html" "--out" "Lcov")

        if [[ "$TEST_MODE" == "release" ]]; then
            tarpaulin_args+=("--release")
        fi

        if [[ -n "$TEST_FEATURES" ]]; then
            tarpaulin_args+=("--features" "$TEST_FEATURES")
        fi

        if [[ -n "$TEST_FILTER" ]]; then
            tarpaulin_args+=("--test" "$TEST_FILTER")
        fi

        cd "$PROJECT_ROOT"
        if ! cargo tarpaulin "${tarpaulin_args[@]}"; then
            error "Coverage tests failed"
            exit 1
        fi

        success "Coverage report generated in target/tarpaulin/"
        info "Open target/tarpaulin/tarpaulin-report.html to view coverage"

    # Check for grcov (requires rustc with coverage enabled)
    elif command -v grcov &> /dev/null; then
        warn "grcov found but requires RUSTFLAGS for coverage"
        warn "Set: export RUSTFLAGS='-C instrument-coverage'"
        warn "Then run: cargo test --workspace"
        warn "Then run: grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/coverage/"
        exit 1

    else
        error "No coverage tool found"
        error "Install cargo-tarpaulin: cargo install cargo-tarpaulin"
        error "Or install grcov: cargo install grcov"
        exit 1
    fi
}

# Run tests normally
run_tests() {
    local cargo_args=()

    # Set test mode
    if [[ "$TEST_MODE" == "release" ]]; then
        cargo_args+=("--release")
        info "Running tests in release mode..."
    else
        info "Running tests in debug mode..."
    fi

    # Set workspace
    if [[ "$TEST_WORKSPACE" == true ]]; then
        cargo_args+=("--workspace")
        info "Testing all workspace members..."
    fi

    # Add features if specified
    if [[ -n "$TEST_FEATURES" ]]; then
        cargo_args+=("--features" "$TEST_FEATURES")
        info "Using features: $TEST_FEATURES"
    fi

    # Add nocapture if requested
    if [[ "$TEST_NOCAPTURE" == true ]]; then
        cargo_args+=("--" "--nocapture")
    fi

    # Add exact match if requested
    if [[ "$TEST_EXACT" == true ]]; then
        cargo_args+=("--" "--exact")
    fi

    # Change to project root
    cd "$PROJECT_ROOT"

    # Add test filter at the end (cargo test -- filter)
    if [[ -n "$TEST_FILTER" ]]; then
        cargo_args+=("--" "$TEST_FILTER")
        info "Running tests matching: $TEST_FILTER"
    fi

    # Run cargo test
    info "Running: cargo test ${cargo_args[*]}"
    if ! cargo test "${cargo_args[@]}"; then
        error "Tests failed"
        exit 1
    fi

    success "All tests passed!"
}

# Main entry point
main() {
    parse_args "$@"

    if [[ "$TEST_COVERAGE" == true ]]; then
        run_coverage
    else
        run_tests
    fi
}

main "$@"
