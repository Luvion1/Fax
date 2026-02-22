#!/bin/bash
# Fax Compiler Coverage Report Generator
# ======================================
# This script generates code coverage reports for the Fax Compiler workspace.
#
# Usage:
#   ./coverage-report.sh          # Generate HTML report
#   ./coverage-report.sh --lcov   # Generate LCOV report for CI
#   ./coverage-report.sh --help   # Show this help message
#
# Requirements:
#   - cargo-llvm-cov: cargo install cargo-llvm-cov
#
# Output:
#   - HTML Report: faxc/coverage-report/html/index.html
#   - LCOV Report: faxc/coverage/lcov.info
#   - Cobertura:   faxc/coverage/cobertura.xml

set -e

# Configuration
WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COVERAGE_DIR="${WORKSPACE_DIR}/coverage"
HTML_REPORT_DIR="${COVERAGE_DIR}/html"
LCOV_OUTPUT="${COVERAGE_DIR}/lcov.info"
CARGO_LLVM_COV_VERSION="0.6"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Coverage thresholds
COVERAGE_THRESHOLD=80
CRITICAL_CRATES=("faxc-drv" "fgc" "faxc-util")
CRITICAL_THRESHOLD=85

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

show_help() {
    head -20 "$0" | tail -17
    exit 0
}

check_dependencies() {
    log_info "Checking dependencies..."
    
    if ! command -v cargo-llvm-cov &> /dev/null; then
        log_warning "cargo-llvm-cov not found. Installing..."
        cargo install cargo-llvm-cov --version "${CARGO_LLVM_COV_VERSION}"
    fi
    
    log_success "Dependencies OK"
}

generate_html_report() {
    log_info "Generating HTML coverage report..."
    
    cd "${WORKSPACE_DIR}"
    
    # Run tests with coverage
    cargo llvm-cov --workspace --all-targets --html --output-dir "${HTML_REPORT_DIR}"
    
    log_success "HTML report generated at: ${HTML_REPORT_DIR}/index.html"
}

generate_lcov_report() {
    log_info "Generating LCOV coverage report..."
    
    cd "${WORKSPACE_DIR}"
    
    # Create coverage directory
    mkdir -p "${COVERAGE_DIR}"
    
    # Run tests with coverage and generate LCOV
    cargo llvm-cov --workspace --all-targets --lcov --output-path "${LCOV_OUTPUT}"
    
    log_success "LCOV report generated at: ${LCOV_OUTPUT}"
}

generate_cobertura_report() {
    log_info "Generating Cobertura coverage report..."
    
    cd "${WORKSPACE_DIR}"
    
    # Run tests with coverage and generate Cobertura
    cargo llvm-cov --workspace --all-targets --cobertura --output-path "${COVERAGE_DIR}/cobertura.xml"
    
    log_success "Cobertura report generated at: ${COVERAGE_DIR}/cobertura.xml"
}

check_coverage_threshold() {
    log_info "Checking coverage thresholds..."
    
    cd "${WORKSPACE_DIR}"
    
    # Get coverage summary
    local summary=$(cargo llvm-cov --workspace --all-targets --summary-only 2>&1)
    
    # Extract line coverage percentage
    local line_coverage=$(echo "$summary" | grep -oP 'LINE\s+\K[\d.]+' | head -1)
    
    if [[ -z "$line_coverage" ]]; then
        log_warning "Could not extract coverage percentage"
        return 0
    fi
    
    # Compare with threshold (using bc for floating point comparison)
    local meets_threshold=$(echo "$line_coverage >= $COVERAGE_THRESHOLD" | bc -l)
    
    if [[ "$meets_threshold" -eq 1 ]]; then
        log_success "Line coverage (${line_coverage}%) meets threshold (${COVERAGE_THRESHOLD}%)"
    else
        log_error "Line coverage (${line_coverage}%) below threshold (${COVERAGE_THRESHOLD}%)"
        return 1
    fi
}

generate_summary() {
    log_info "Generating coverage summary..."
    
    cd "${WORKSPACE_DIR}"
    
    # Generate summary report
    cargo llvm-cov --workspace --all-targets --summary-only
    
    # Generate per-crate breakdown
    echo ""
    echo "=== Per-Crate Coverage ==="
    cargo llvm-cov --workspace --all-targets --fail-under-lines ${COVERAGE_THRESHOLD}
}

# Main execution
main() {
    local mode="html"
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --lcov)
                mode="lcov"
                shift
                ;;
            --cobertura)
                mode="cobertura"
                shift
                ;;
            --summary)
                mode="summary"
                shift
                ;;
            --all)
                mode="all"
                shift
                ;;
            --help|-h)
                show_help
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                ;;
        esac
    done
    
    log_info "Fax Compiler Coverage Report Generator"
    log_info "======================================="
    
    # Check dependencies
    check_dependencies
    
    # Generate reports based on mode
    case $mode in
        html)
            generate_html_report
            ;;
        lcov)
            generate_lcov_report
            ;;
        cobertura)
            generate_cobertura_report
            ;;
        summary)
            generate_summary
            ;;
        all)
            generate_html_report
            generate_lcov_report
            generate_cobertura_report
            generate_summary
            check_coverage_threshold
            ;;
    esac
    
    log_success "Coverage report generation complete!"
}

# Run main function
main "$@"