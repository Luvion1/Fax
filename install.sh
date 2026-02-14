#!/bin/bash
# Fax Installer Script
# Inspired by rustup - The Rust Installer
# Usage: curl --proto '=https' --tlsv1.2 -sSf https://luvion1.github.io/Fax/install.sh | sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
FAX_VERSION="0.1.0"
FAX_REPO="https://github.com/Luvion1/Fax"
INSTALL_DIR="${FAX_INSTALL:-$HOME/.fax}"
BIN_DIR="$INSTALL_DIR/bin"

# Print functions
say() {
    echo -e "${GREEN}fax:${NC} $1"
}

err() {
    echo -e "${RED}error:${NC} $1" >&2
}

warn() {
    echo -e "${YELLOW}warning:${NC} $1"
}

need_cmd() {
    if ! check_cmd "$1"; then
        err "need '$1' (command not found)"
        exit 1
    fi
}

check_cmd() {
    command -v "$1" > /dev/null 2>&1
}

# Detect architecture and platform
detect_platform() {
    local _ostype _cputype
    
    _ostype="$(uname -s)"
    _cputype="$(uname -m)"
    
    case "$_ostype" in
        Linux)
            _ostype=linux
            ;;
        Darwin)
            _ostype=darwin
            ;;
        *)
            err "unsupported OS: $_ostype"
            exit 1
            ;;
    esac
    
    case "$_cputype" in
        x86_64 | x86-64 | x64 | amd64)
            _cputype=x86_64
            ;;
        aarch64 | arm64)
            _cputype=aarch64
            ;;
        *)
            err "unsupported architecture: $_cputype"
            exit 1
            ;;
    esac
    
    echo "${_cputype}-${_ostype}"
}

# Check if dependencies are installed
check_dependencies() {
    say "Checking dependencies..."
    
    local deps_missing=()
    
    # Check for git
    if ! check_cmd git; then
        deps_missing+=("git")
    fi
    
    # Check for Python
    if ! check_cmd python3; then
        deps_missing+=("python3")
    fi
    
    # Check for Node.js
    if ! check_cmd node; then
        deps_missing+=("nodejs")
    fi
    
    # Check for Rust
    if ! check_cmd rustc; then
        deps_missing+=("rust")
    fi
    
    # Check for Zig
    if ! check_cmd zig; then
        deps_missing+=("zig")
    fi
    
    # Check for GHC
    if ! check_cmd ghc; then
        deps_missing+=("ghc")
    fi
    
    if [ ${#deps_missing[@]} -ne 0 ]; then
        err "Missing dependencies: ${deps_missing[*]}"
        echo
        echo "Please install the missing dependencies:"
        echo
        echo "macOS (Homebrew):"
        echo "  brew install ${deps_missing[*]}"
        echo
        echo "Ubuntu/Debian:"
        for dep in "${deps_missing[@]}"; do
            case "$dep" in
                rust)
                    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                    ;;
                zig)
                    echo "  sudo snap install zig --classic"
                    ;;
                ghc)
                    echo "  sudo apt-get install ghc"
                    ;;
                nodejs)
                    echo "  curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -"
                    echo "  sudo apt-get install -y nodejs"
                    ;;
                *)
                    echo "  sudo apt-get install $dep"
                    ;;
            esac
        done
        exit 1
    fi
    
    say "All dependencies satisfied!"
}

# Download and install Fax
install_fax() {
    local _platform="$(detect_platform)"
    say "Detected platform: $_platform"
    say "Installing Fax to $INSTALL_DIR..."
    
    # Create directories
    mkdir -p "$BIN_DIR"
    mkdir -p "$INSTALL_DIR/lib"
    mkdir -p "$INSTALL_DIR/share/fax"
    
    # Clone repository
    if [ -d "$INSTALL_DIR/repo" ]; then
        say "Updating existing installation..."
        (cd "$INSTALL_DIR/repo" && git pull)
    else
        say "Cloning Fax repository..."
        git clone --depth 1 "$FAX_REPO.git" "$INSTALL_DIR/repo"
    fi
    
    # Install Node.js dependencies
    say "Installing Node.js dependencies..."
    (cd "$INSTALL_DIR/repo" && npm install)
    
    # Build compiler components
    say "Building compiler components (this may take a few minutes)..."
    (cd "$INSTALL_DIR/repo" && make build)
    
    # Create wrapper scripts
    create_wrappers
    
    # Create version file
    echo "$FAX_VERSION" > "$INSTALL_DIR/share/fax/version"
    
    say "Fax $FAX_VERSION installed successfully!"
}

# Create wrapper scripts
create_wrappers() {
    say "Creating command wrappers..."
    
    # Create faxt wrapper
    cat > "$BIN_DIR/faxt" << 'EOF'
#!/bin/bash
set -e

FAX_DIR="${FAX_INSTALL:-$HOME/.fax}"
exec python3 "$FAX_DIR/repo/faxt/main.py" "$@"
EOF
    chmod +x "$BIN_DIR/faxt"
    
    # Create fax wrapper (shorthand)
    ln -sf "$BIN_DIR/faxt" "$BIN_DIR/fax"
    
    # Create faxc wrapper
    cat > "$BIN_DIR/faxc" << 'EOF'
#!/bin/bash
set -e

FAX_DIR="${FAX_INSTALL:-$HOME/.fax}"
exec python3 "$FAX_DIR/repo/faxc/main.py" "$@"
EOF
    chmod +x "$BIN_DIR/faxc"
}

# Add to shell configuration
add_to_path() {
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
    
    # Check if already in PATH
    if [[ ":$PATH:" == *":$BIN_DIR:"* ]]; then
        say "$BIN_DIR is already in PATH"
        return
    fi
    
    # Add to shell config
    if [ -f "$_shell_rc" ]; then
        echo "" >> "$_shell_rc"
        echo "# Fax programming language" >> "$_shell_rc"
        echo 'export PATH="$HOME/.fax/bin:$PATH"' >> "$_shell_rc"
        say "Added $BIN_DIR to PATH in $_shell_rc"
        warn "Please restart your shell or run: source $_shell_rc"
    else
        warn "Could not find shell config file. Please add $BIN_DIR to your PATH manually"
        echo "Add this line to your shell config:"
        echo 'export PATH="$HOME/.fax/bin:$PATH"'
    fi
}

# Post-installation instructions
post_install() {
    echo
    say "Installation complete! 🎉"
    echo
    echo "To get started:"
    echo "  1. Restart your terminal or run: source ~/.bashrc (or ~/.zshrc)"
    echo "  2. Verify installation: faxt --version"
    echo "  3. Create your first program:"
    echo
    echo "     echo 'fn main() { print(\"Hello, Fax!\"); }' > hello.fax"
    echo "     faxt run hello.fax"
    echo
    echo "Documentation: https://luvion1.github.io/Fax/"
    echo "Repository: $FAX_REPO"
    echo
}

# Main installation flow
main() {
    echo
    say "Fax Installer"
    say "============="
    say "Version: $FAX_VERSION"
    say "Install directory: $INSTALL_DIR"
    echo
    
    # Check dependencies
    check_dependencies
    echo
    
    # Install
    install_fax
    echo
    
    # Add to PATH
    add_to_path
    echo
    
    # Post-install
    post_install
}

# Run main function
main
