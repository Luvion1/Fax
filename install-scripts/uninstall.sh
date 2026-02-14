#!/bin/bash
# Fax Uninstaller Script
# Usage: ~/.fax/bin/fax-uninstall

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

INSTALL_DIR="${FAX_INSTALL:-$HOME/.fax}"

say() {
    echo -e "${GREEN}fax:${NC} $1"
}

err() {
    echo -e "${RED}error:${NC} $1" >&2
}

warn() {
    echo -e "${YELLOW}warning:${NC} $1"
}

# Remove from shell configs
remove_from_path() {
    local _shell_rc
    
    case "$(basename "$SHELL")" in
        bash)
            _shell_rc="$HOME/.bashrc"
            ;;
        zsh)
            _shell_rc="$HOME/.zshrc"
            ;;
        fish)
            _shell_rc="$HOME/.config/fish/config.fish"
            ;;
        *)
            _shell_rc="$HOME/.profile"
            ;;
    esac
    
    if [ -f "$_shell_rc" ]; then
        # Remove Fax-related lines
        sed -i.bak '/# Fax programming language/d' "$_shell_rc"
        sed -i.bak '/\.fax\/bin/d' "$_shell_rc"
        rm -f "$_shell_rc.bak"
        say "Removed Fax from $_shell_rc"
    fi
}

# Main uninstall
main() {
    echo
    say "Fax Uninstaller"
    say "==============="
    echo
    
    if [ ! -d "$INSTALL_DIR" ]; then
        err "Fax is not installed at $INSTALL_DIR"
        exit 1
    fi
    
    warn "This will remove: $INSTALL_DIR"
    read -p "Are you sure? [y/N] " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        say "Uninstall cancelled"
        exit 0
    fi
    
    # Remove from PATH
    remove_from_path
    
    # Remove installation directory
    say "Removing $INSTALL_DIR..."
    rm -rf "$INSTALL_DIR"
    
    say "Fax has been uninstalled"
    echo
    warn "Please restart your shell for changes to take effect"
}

main
