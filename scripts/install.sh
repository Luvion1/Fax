#!/bin/bash
# =============================================================================
# Fax Compiler - Install Dependencies
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[!]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }

# =============================================================================
# OS Detection
# =============================================================================

detect_os() {
    local os
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    case "$os" in
        linux*)     [ -f /etc/os-release ] && source /etc/os-release && echo "$ID" || echo "linux" ;;
        darwin*)    echo "macos" ;;
        mingw*|msys*|cygwin*) echo "windows" ;;
        *)          echo "unknown" ;;
    esac
}

has_command() { command -v "$1" &>/dev/null; }

# =============================================================================
# Install
# =============================================================================

install_llvm_ubuntu() {
    log_info "Installing LLVM 20..."
    [ ! -f /etc/apt/sources.list.d/llvm.list ] && {
        wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | sudo tee /etc/apt/trusted.gpg.d/apt.llvm.asc &>/dev/null || true
        echo "deb http://apt.llvm.org/$(lsb_release -sc)/ llvm-toolchain-$(lsb_release -sc)-20 main" | sudo tee /etc/apt/sources.list.d/llvm.list &>/dev/null || true
    }
    sudo apt-get update -qq
    sudo apt-get install -y llvm-20 llvm-20-dev llvm-20-tools libpolly-20-dev clang-20 lld-20 libzstd-dev
    log_success "LLVM 20 installed!"
}

install_llvm_fedora() {
    log_info "Installing LLVM 20..."
    sudo dnf install -y llvm20-devel llvm20-static clang20 lld20 zstd-devel
    log_success "LLVM 20 installed!"
}

install_llvm_arch() {
    log_info "Installing LLVM..."
    sudo pacman -Sy --noconfirm llvm clang lld polly zstd
    log_success "LLVM installed!"
}

install_llvm_macos() {
    log_info "Installing LLVM 20..."
    brew install llvm@20
    log_success "LLVM 20 installed!"
}

install_rust() {
    log_info "Checking Rust..."
    if has_command rustc; then
        log_success "Rust: $(rustc --version | awk '{print $2}')"
        rustup update stable 2>/dev/null || true
    else
        log_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    rustup component add rustfmt clippy 2>/dev/null || true
    log_success "Rust ready!"
}

configure_env() {
    for path in /usr/lib/llvm-20 /usr/lib64/llvm20 /opt/llvm@20; do
        if [ -d "$path" ]; then
            export LLVM_SYS_20_PREFIX="$path"
            for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
                [ -f "$rc" ] && ! grep -q LLVM_SYS_20_PREFIX "$rc" && echo "export LLVM_SYS_20_PREFIX=$path" >> "$rc"
            done
            break
        fi
    done
}

verify() {
    log_info "Verifying..."
    has_command rustc && log_success "Rust: OK"
    if [ -n "${LLVM_SYS_20_PREFIX:-}" ] && [ -f "$LLVM_SYS_20_PREFIX/bin/llvm-config" ]; then
        log_success "LLVM: OK"
    elif has_command llvm-config-20; then
        log_success "LLVM: OK"
    else
        log_warn "LLVM not found in PATH"
    fi
}

# =============================================================================
# Main
# =============================================================================

show_help() {
    cat << EOF
${BOLD}Fax Compiler - Install Dependencies${NC}

Usage:
    $0 [OPTIONS]

Options:
    -h, --help         Show help
    --rust-only        Only Rust
    --llvm-only        Only LLVM
    --check            Check installation
    --verify           Verify after install

EOF
}

main() {
    local install_rust=true install_llvm=true verify=false check_only=false
    
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h|--help) show_help; exit 0 ;;
            --rust-only) install_llvm=false ;;
            --llvm-only) install_rust=false ;;
            --check|--verify) verify=true ;;
            --skip-rust) install_rust=false ;;
            --skip-llvm) install_llvm=false ;;
            *) ;;
        esac
        shift
    done
    
    echo ""
    echo -e "${BOLD}========================================${NC}"
    echo -e "${BOLD}  Fax Compiler - Install${NC}"
    echo -e "${BOLD}========================================${NC}"
    echo ""
    
    local os=$(detect_os)
    log_info "Detected: $os"
    echo ""
    
    [ "$verify" = true ] && verify && exit 0
    
    if [ "$install_llvm" = true ]; then
        case "$os" in
            ubuntu|debian|linuxmint|pop) install_llvm_ubuntu ;;
            fedora|rhel|centos|rocky|alma) install_llvm_fedora ;;
            arch|manjaro) install_llvm_arch ;;
            macos) install_llvm_macos ;;
            windows) log_error "Windows: download LLVM from llvm.org" ;;
            *) log_error "Unknown OS" ;;
        esac
    fi
    
    [ "$install_rust" = true ] && install_rust
    configure_env
    [ "$verify" = true ] && verify
    
    echo ""
    log_success "Done! Run: ./scripts/build.sh"
}

main "$@"
