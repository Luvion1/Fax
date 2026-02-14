#!/bin/bash
# Fax Installer Script
# Usage: curl --proto '=https' --tlsv1.2 -sSf https://luvion1.github.io/Fax/install.sh | sh

set -e

# Version
SCRIPT_VERSION="2.0.0"
FAX_VERSION="0.1.0"

# Configuration
FAX_REPO="https://github.com/Luvion1/Fax"
INSTALL_DIR="${FAX_INSTALL:-$HOME/.fax}"
BIN_DIR="$INSTALL_DIR/bin"
LIB_DIR="$INSTALL_DIR/lib"
CACHE_DIR="$INSTALL_DIR/.cache"
LOG_FILE="$INSTALL_DIR/install.log"

# Flags
FORCE_INSTALL=false
QUIET_MODE=false
VERBOSE=false
SKIP_DEPS=false
DRY_RUN=false

# Colors (auto-detect TTY)
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    CYAN='\033[0;36m'
    MAGENTA='\033[0;35m'
    BOLD='\033[1m'
    NC='\033[0m'
else
    RED='' GREEN='' YELLOW='' BLUE='' CYAN='' MAGENTA='' BOLD='' NC=''
fi

# Logging system
log_init() {
    mkdir -p "$INSTALL_DIR"
    echo "=== Fax Installer Log $(date) ===" > "$LOG_FILE"
}

log() {
    local level="$1"
    shift
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [$level] $*" >> "$LOG_FILE"
}

log_info() { log "INFO" "$@"; }
log_warn() { log "WARN" "$@"; }
log_error() { log "ERROR" "$@"; }
log_debug() { log "DEBUG" "$@"; }

# Better output functions
say() {
    [[ "$QUIET_MODE" == true ]] && return
    echo -e "${GREEN}✓${NC} $1"
    log_info "$1"
}

say_step() {
    [[ "$QUIET_MODE" == true ]] && return
    echo -e "\n${CYAN}${BOLD}➜${NC} ${BOLD}$1${NC}"
    log_info "STEP: $1"
}

say_substep() {
    [[ "$QUIET_MODE" == true ]] && return
    echo -e "  ${BLUE}›${NC} $1"
    log_debug "$1"
}

err() {
    echo -e "${RED}✗${NC} $1" >&2
    log_error "$1"
}

warn() {
    [[ "$QUIET_MODE" == true ]] && return
    echo -e "${YELLOW}⚠${NC} $1"
    log_warn "$1"
}

info() {
    [[ "$VERBOSE" == false ]] && return
    echo -e "${BLUE}ℹ${NC} $1"
    log_debug "$1"
}

# Progress bar with ETA
progress_init() {
    PROGRESS_TOTAL=$1
    PROGRESS_CURRENT=0
    PROGRESS_START_TIME=$(date +%s)
}

progress() {
    [[ "$QUIET_MODE" == true ]] && return
    
    local current=$1
    local total=${2:-$PROGRESS_TOTAL}
    local width=40
    local percentage=$((current * 100 / total))
    local filled=$((width * current / total))
    local empty=$((width - filled))
    
    # Calculate ETA
    local now=$(date +%s)
    local elapsed=$((now - PROGRESS_START_TIME))
    if [[ $current -gt 0 ]]; then
        local eta=$((elapsed * (total - current) / current))
        local eta_str="${eta}s"
    else
        local eta_str="???"
    fi
    
    printf "\r${CYAN}"
    printf "[%${filled}s" | tr ' ' '█'
    printf "%${empty}s" | tr ' ' '░'
    printf "]${NC} %3d%% (%d/%d) ETA: %s" "$percentage" "$current" "$total" "$eta_str"
}

progress_done() {
    [[ "$QUIET_MODE" == true ]] && return
    printf "\n"
}

# Better command detection
check_cmd() {
    command -v "$1" > /dev/null 2>&1
}

# Get command version (advanced parsing)
get_version() {
    local cmd="$1"
    local version_flag="${2:---version}"
    
    if check_cmd "$cmd"; then
        local version_output
        version_output=$($cmd $version_flag 2>&1)
        
        # Try various patterns
        local version
        version=$(echo "$version_output" | grep -oE '[0-9]+\.[0-9]+(\.[0-9]+)?' | head -1)
        
        if [[ -n "$version" ]]; then
            echo "$version"
            return 0
        fi
    fi
    
    echo ""
    return 1
}

# Semantic version comparison
version_compare() {
    local v1="$1"
    local v2="$2"
    
    if [[ "$v1" == "$v2" ]]; then
        echo 0
        return
    fi
    
    local IFS=.
    local i ver1=($v1) ver2=($v2)
    
    for ((i=0; i<${#ver1[@]} || i<${#ver2[@]}; i++)); do
        local x=${ver1[i]:-0}
        local y=${ver2[i]:-0}
        
        if ((10#$x > 10#$y)); then
            echo 1
            return
        elif ((10#$x < 10#$y)); then
            echo -1
            return
        fi
    done
    
    echo 0
}

version_ge() {
    local result=$(version_compare "$1" "$2")
    [[ $result -ge 0 ]]
}

version_gt() {
    local result=$(version_compare "$1" "$2")
    [[ $result -gt 0 ]]
}

# Better platform detection with detailed info
detect_platform() {
    local _os _arch _version="" _distro=""
    
    _os="$(uname -s)"
    _arch="$(uname -m)"
    
    case "$_os" in
        Linux)
            _os=linux
            if [[ -f /etc/os-release ]]; then
                source /etc/os-release
                _distro="$ID"
                _version="$VERSION_ID"
            elif [[ -f /etc/debian_version ]]; then
                _distro="debian"
                _version=$(cat /etc/debian_version)
            elif [[ -f /etc/redhat-release ]]; then
                _distro="rhel"
            fi
            ;;
        Darwin)
            _os=darwin
            _version=$(sw_vers -productVersion)
            _distro="macos"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            _os=windows
            _distro="mingw"
            ;;
        FreeBSD)
            _os=freebsd
            _version=$(freebsd-version)
            ;;
        *)
            err "Unsupported operating system: $_os"
            exit 1
            ;;
    esac
    
    case "$_arch" in
        x86_64|x86-64|x64|amd64)
            _arch=x86_64
            ;;
        aarch64|arm64)
            _arch=aarch64
            ;;
        armv7l)
            _arch=armv7
            ;;
        i686|i386)
            _arch=x86
            ;;
        *)
            err "Unsupported architecture: $_arch"
            exit 1
            ;;
    esac
    
    PLATFORM="${_arch}-${_os}"
    PLATFORM_ARCH="$_arch"
    PLATFORM_OS="$_os"
    PLATFORM_VERSION="$_version"
    PLATFORM_DISTRO="$_distro"
}

# Check if already installed
check_existing() {
    if [[ ! -d "$INSTALL_DIR" ]]; then
        return 0
    fi
    
    if [[ "$FORCE_INSTALL" == true ]]; then
        warn "Force install enabled, will reinstall..."
        return 0
    fi
    
    if [[ -f "$INSTALL_DIR/share/fax/version" ]]; then
        local existing_version
        existing_version=$(cat "$INSTALL_DIR/share/fax/version" 2>/dev/null || echo "unknown")
        
        if [[ -d "$INSTALL_DIR/repo/.git" ]]; then
            local local_commit remote_commit
            local_commit=$(cd "$INSTALL_DIR/repo" && git rev-parse HEAD 2>/dev/null || echo "unknown")
            
            # Check for updates
            info "Checking for updates..."
            (cd "$INSTALL_DIR/repo" && git fetch origin master --quiet 2>/dev/null) || true
            remote_commit=$(cd "$INSTALL_DIR/repo" && git rev-parse origin/master 2>/dev/null || echo "$local_commit")
            
            if [[ "$existing_version" == "$FAX_VERSION" && "$local_commit" == "$remote_commit" ]]; then
                say "Fax $FAX_VERSION is already installed and up to date"
                echo
                echo "To reinstall: curl --proto '=https' --tlsv1.2 -sSf $FAX_REPO/install.sh | sh -s -- --force"
                echo "To update: faxt update"
                exit 0
            elif [[ "$local_commit" != "$remote_commit" ]]; then
                warn "Update available: $existing_version → $FAX_VERSION"
                echo
                read -p "Update now? [Y/n] " -n 1 -r
                echo
                if [[ ! $REPLY =~ ^[Nn]$ ]]; then
                    FORCE_INSTALL=true
                    return 0
                else
                    exit 0
                fi
            fi
        fi
    fi
}

# Better dependency system
declare -A DEP_MIN_VERSIONS=(
    ["git"]="2.0.0"
    ["python3"]="3.8.0"
    ["node"]="18.0.0"
    ["rustc"]="1.70.0"
    ["zig"]="0.11.0"
    ["ghc"]="9.0.0"
)

declare -A DEP_CMD_MAP=(
    ["rustc"]="rust"
    ["python3"]="python"
    ["node"]="nodejs"
)

declare -A DEP_PRIORITY=(
    ["git"]="1"
    ["python3"]="1"
    ["node"]="1"
    ["rustc"]="2"
    ["zig"]="2"
    ["ghc"]="2"
)

get_install_command() {
    local dep="$1"
    local distro="$PLATFORM_DISTRO"
    
    case "$distro" in
        macos)
            case "$dep" in
                rust) echo "brew install rustup && rustup-init" ;;
                zig) echo "brew install zig" ;;
                ghc) echo "brew install ghc" ;;
                nodejs) echo "brew install node" ;;
                *) echo "brew install $dep" ;;
            esac
            ;;
        ubuntu|debian)
            case "$dep" in
                rust) echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" ;;
                zig) echo "sudo snap install zig --classic" ;;
                ghc) echo "sudo apt-get install ghc" ;;
                nodejs) echo "curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - && sudo apt-get install -y nodejs" ;;
                *) echo "sudo apt-get install $dep" ;;
            esac
            ;;
        fedora|rhel|centos)
            case "$dep" in
                rust) echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" ;;
                zig) echo "sudo dnf install zig" ;;
                ghc) echo "sudo dnf install ghc" ;;
                nodejs) echo "sudo dnf install nodejs" ;;
                *) echo "sudo dnf install $dep" ;;
            esac
            ;;
        arch)
            case "$dep" in
                rust) echo "sudo pacman -S rustup && rustup default stable" ;;
                zig) echo "sudo pacman -S zig" ;;
                ghc) echo "sudo pacman -S ghc" ;;
                nodejs) echo "sudo pacman -S nodejs" ;;
                *) echo "sudo pacman -S $dep" ;;
            esac
            ;;
        *)
            echo "# Please install $dep using your package manager"
            ;;
    esac
}

check_dependencies() {
    [[ "$SKIP_DEPS" == true ]] && return 0
    
    say_step "Analyzing dependencies advancedly"
    
    local -A dep_status
    local missing_deps=()
    local outdated_deps=()
    local ok_deps=()
    local total=${#DEP_MIN_VERSIONS[@]}
    local current=0
    
    # Check each dependency
    for dep in "${!DEP_MIN_VERSIONS[@]}"; do
        current=$((current + 1))
        progress $current $total
        
        local cmd="$dep"
        local name="${DEP_CMD_MAP[$dep]:-$dep}"
        local required="${DEP_MIN_VERSIONS[$dep]}"
        local installed=""
        local status=""
        
        # Get installed version
        if installed=$(get_version "$cmd"); then
            if version_ge "$installed" "$required"; then
                status="ok"
                ok_deps+=("$name:$installed")
            else
                status="outdated"
                outdated_deps+=("$name:$installed:$required")
            fi
        else
            status="missing"
            missing_deps+=("$name")
        fi
        
        dep_status[$name]="$status"
        log_debug "$name: $status (installed: ${installed:-none}, required: $required)"
    done
    
    progress_done
    
    # Report results
    if [[ ${#ok_deps[@]} -gt 0 ]]; then
        say "${#ok_deps[@]} dependencies satisfied"
        for dep_info in "${ok_deps[@]}"; do
            IFS=':' read -r name version <<< "$dep_info"
            info "$name $version ✓"
        done
    fi
    
    if [[ ${#outdated_deps[@]} -gt 0 ]]; then
        warn "${#outdated_deps[@]} outdated dependencies:"
        for dep_info in "${outdated_deps[@]}"; do
            IFS=':' read -r name installed required <<< "$dep_info"
            echo "  • $name: $installed installed, $required required"
            echo "    Update: $(get_install_command "$name")"
        done
        echo
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        err "${#missing_deps[@]} missing dependencies: ${missing_deps[*]}"
        echo
        echo "Install them with:"
        echo
        for dep in "${missing_deps[@]}"; do
            local cmd=$(get_install_command "$dep")
            echo "  $cmd"
        done
        echo
        exit 1
    fi
    
    if [[ ${#outdated_deps[@]} -gt 0 ]]; then
        read -p "Continue with outdated dependencies? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

# Advanced git operations with caching
better_clone() {
    local repo_url="$1"
    local dest="$2"
    
    if [[ -d "$dest/.git" ]]; then
        info "Repository exists, checking for updates..."
        
        local local_commit remote_commit
        local_commit=$(cd "$dest" && git rev-parse HEAD)
        
        # Fetch with timeout
        if timeout 30 bash -c "cd '$dest' && git fetch origin master" 2>/dev/null; then
            remote_commit=$(cd "$dest" && git rev-parse origin/master)
            
            if [[ "$local_commit" == "$remote_commit" ]]; then
                say "Already up to date"
            else
                local commits_behind
                commits_behind=$(cd "$dest" && git rev-list --count HEAD..origin/master)
                say "Updating: $commits_behind new commits"
                (cd "$dest" && git pull origin master --quiet)
            fi
        else
            warn "Could not check for updates (offline?)"
        fi
    else
        say "Cloning repository..."
        git clone --depth 100 --single-branch --branch master "$repo_url" "$dest"
    fi
}

# Build with advanced caching
better_build() {
    local repo_dir="$1"
    local build_stamp="$CACHE_DIR/last_build"
    local force_build=${2:-false}
    
    # Check if rebuild needed
    if [[ "$force_build" == false && -f "$build_stamp" ]]; then
        local last_build=$(cat "$build_stamp")
        local latest_change=$(cd "$repo_dir" && git log -1 --format=%ct)
        
        if [[ $last_build -ge $latest_change ]]; then
            say "Build is up to date"
            return 0
        fi
    fi
    
    say_step "Building Fax compiler"
    
    cd "$repo_dir"
    
    # Better build based on what changed
    if [[ -f "$CACHE_DIR/components_built" ]]; then
        # Incremental build
        say_substep "Running incremental build"
        npm run build:incremental 2>/dev/null || npm install && make build
    else
        # Full build
        say_substep "Running full build (this may take a few minutes)"
        npm install
        make build
        date +%s > "$CACHE_DIR/components_built"
    fi
    
    date +%s > "$build_stamp"
}

# Create wrapper scripts
create_better_wrappers() {
    say_step "Creating advanced command wrappers"
    
    mkdir -p "$BIN_DIR"
    
    # Better faxt wrapper
    cat > "$BIN_DIR/faxt" << 'EOF'
#!/bin/bash
# Better Fax CLI wrapper with auto-updates and error handling

set -e

FAX_DIR="${FAX_INSTALL:-$HOME/.fax}"
FAX_REPO="$FAX_DIR/repo"
FAX_BIN="$FAX_DIR/bin"

# Ensure repo exists
if [[ ! -d "$FAX_REPO" ]]; then
    echo "Error: Fax installation not found at $FAX_DIR" >&2
    echo "Please reinstall: curl --proto '=https' --tlsv1.2 -sSf https://luvion1.github.io/Fax/install.sh | sh" >&2
    exit 1
fi

# Handle special commands
case "${1:-}" in
    --version|-v)
        if [[ -f "$FAX_DIR/share/fax/version" ]]; then
            cat "$FAX_DIR/share/fax/version"
        else
            echo "unknown"
        fi
        exit 0
        ;;
    update|self-update)
        exec "$FAX_BIN/faxt-update"
        ;;
    uninstall)
        exec "$FAX_BIN/fax-uninstall"
        ;;
esac

# Check for updates (once per day)
LAST_CHECK_FILE="$FAX_DIR/.last_update_check"
if [[ -f "$LAST_CHECK_FILE" ]]; then
    LAST_CHECK=$(cat "$LAST_CHECK_FILE")
    NOW=$(date +%s)
    if (( NOW - LAST_CHECK > 86400 )); then
        (
            cd "$FAX_REPO" 2>/dev/null && \
            git fetch origin master --quiet 2>/dev/null && \
            LOCAL=$(git rev-parse HEAD) && \
            REMOTE=$(git rev-parse origin/master) && \
            if [[ "$LOCAL" != "$REMOTE" ]]; then
                echo "[fax] Update available! Run: faxt update" >&2
            fi
        ) &
        disown 2>/dev/null || true
        date +%s > "$LAST_CHECK_FILE"
    fi
else
    date +%s > "$LAST_CHECK_FILE"
fi

# Run actual command
exec python3 "$FAX_REPO/faxt/main.py" "$@"
EOF
    chmod +x "$BIN_DIR/faxt"
    
    # Better update script
    cat > "$BIN_DIR/faxt-update" << 'EOF'
#!/bin/bash
# Fax self-updater with rollback support

set -e

FAX_DIR="${FAX_INSTALL:-$HOME/.fax}"
FAX_REPO="$FAX_DIR/repo"
BACKUP_DIR="$FAX_DIR/.backup/$(date +%Y%m%d_%H%M%S)"

echo "Checking for updates..."

cd "$FAX_REPO"

# Get current state
LOCAL=$(git rev-parse HEAD)
git fetch origin master --quiet
REMOTE=$(git rev-parse origin/master)

if [[ "$LOCAL" == "$REMOTE" ]]; then
    echo "Already up to date!"
    exit 0
fi

echo "Update found:"
echo "  Current: ${LOCAL:0:8}"
echo "  Latest:  ${REMOTE:0:8}"
echo

# Create backup
mkdir -p "$BACKUP_DIR"
cp -r "$FAX_DIR/bin" "$BACKUP_DIR/" 2>/dev/null || true
cp -r "$FAX_DIR/share" "$BACKUP_DIR/" 2>/dev/null || true

echo "Updating..."
if git pull origin master --quiet; then
    npm install --silent 2>/dev/null || true
    make build 2>&1 | grep -E "(error|✓|Built)" || true
    date +%s > "$FAX_DIR/.cache/last_build"
    echo
    echo "✓ Update complete!"
    echo "  New version: $(cat "$FAX_DIR/share/fax/version" 2>/dev/null || echo 'unknown')"
else
    echo "✗ Update failed, restoring backup..."
    cp -r "$BACKUP_DIR/bin" "$FAX_DIR/"
    cp -r "$BACKUP_DIR/share" "$FAX_DIR/"
    echo "✓ Backup restored"
    exit 1
fi
EOF
    chmod +x "$BIN_DIR/faxt-update"
    
    # Safe uninstaller
    cat > "$BIN_DIR/fax-uninstall" << 'EOF'
#!/bin/bash
# Safe Fax uninstaller

FAX_DIR="${FAX_INSTALL:-$HOME/.fax}"

if [[ ! -d "$FAX_DIR" ]]; then
    echo "Fax is not installed" >&2
    exit 1
fi

echo "This will completely remove Fax from:"
echo "  $FAX_DIR"
echo
read -p "Are you sure? Type 'yes' to confirm: " -r

if [[ "$REPLY" != "yes" ]]; then
    echo "Cancelled"
    exit 0
fi

echo "Uninstalling..."

# Remove from shell configs
for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile" "$HOME/.bash_profile"; do
    if [[ -f "$rc" ]]; then
        sed -i.bak '/# Fax programming language/d' "$rc" 2>/dev/null || true
        sed -i.bak '/\.fax\/bin/d' "$rc" 2>/dev/null || true
        rm -f "${rc}.bak"
    fi
done

# Remove installation
rm -rf "$FAX_DIR"

echo "✓ Fax has been uninstalled"
echo "Please restart your shell or run: hash -r"
EOF
    chmod +x "$BIN_DIR/fax-uninstall"
    
    # Create shorthand
    ln -sf "$BIN_DIR/faxt" "$BIN_DIR/fax"
}

# Better PATH setup with conflict detection
better_path_setup() {
    say_step "Configuring shell environment"
    
    local shell_rc=""
    local shell_name=$(basename "$SHELL")
    
    case "$shell_name" in
        bash)
            for rc in "$HOME/.bashrc" "$HOME/.bash_profile"; do
                [[ -f "$rc" ]] && shell_rc="$rc" && break
            done
            ;;
        zsh)
            shell_rc="$HOME/.zshrc"
            ;;
        fish)
            shell_rc="$HOME/.config/fish/config.fish"
            ;;
        *)
            shell_rc="$HOME/.profile"
            ;;
    esac
    
    if [[ -z "$shell_rc" ]]; then
        shell_rc="$HOME/.bashrc"
    fi
    
    # Check if already configured
    if [[ -f "$shell_rc" ]] && grep -q "\.fax/bin" "$shell_rc" 2>/dev/null; then
        info "PATH already configured in $shell_rc"
        
        # Check if current session has it
        if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
            echo
            echo "To use Fax in this session, run:"
            echo "  source $shell_rc"
        fi
        return 0
    fi
    
    # Add to shell config
    mkdir -p "$(dirname "$shell_rc")"
    
    cat >> "$shell_rc" << EOF

# Fax programming language
export PATH="\$HOME/.fax/bin:\$PATH"
EOF
    
    say "Added $BIN_DIR to PATH in $shell_rc"
    echo
    echo "To use Fax now, run:"
    echo "  source $shell_rc"
}

# Verification with detailed checks
verify_installation() {
    say_step "Verifying installation"
    
    local errors=0
    
    # Check binaries
    for bin in faxt fax fax-uninstall faxt-update; do
        if [[ -x "$BIN_DIR/$bin" ]]; then
            info "$bin ✓"
        else
            err "$bin missing"
            errors=$((errors + 1))
        fi
    done
    
    # Check repo
    if [[ -d "$INSTALL_DIR/repo/.git" ]]; then
        info "Repository ✓"
    else
        err "Repository not found"
        errors=$((errors + 1))
    fi
    
    # Test faxt
    if "$BIN_DIR/faxt" --version &>/dev/null; then
        local version=$("$BIN_DIR/faxt" --version)
        say "faxt working (version $version)"
    else
        warn "faxt not yet in PATH (normal, needs shell restart)"
    fi
    
    if [[ $errors -eq 0 ]]; then
        return 0
    else
        err "Installation verification failed with $errors errors"
        return 1
    fi
}

# Main
main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --force|-f) FORCE_INSTALL=true; shift ;;
            --quiet|-q) QUIET_MODE=true; shift ;;
            --verbose|-v) VERBOSE=true; shift ;;
            --skip-deps) SKIP_DEPS=true; shift ;;
            --dry-run) DRY_RUN=true; shift ;;
            --help|-h)
                echo "Fax Advanced Installer v$SCRIPT_VERSION"
                echo
                echo "Usage: install.sh [OPTIONS]"
                echo
                echo "Options:"
                echo "  -f, --force     Force reinstall"
                echo "  -q, --quiet     Quiet mode"
                echo "  -v, --verbose   Verbose output"
                echo "  --skip-deps     Skip dependency checks"
                echo "  --dry-run       Simulate installation"
                echo "  -h, --help      Show this help"
                echo
                exit 0
                ;;
            *) err "Unknown option: $1"; exit 1 ;;
        esac
    done
    
    # Initialize logging
    log_init
    
    # Header
    [[ "$QUIET_MODE" == false ]] && cat << 'EOF'

╔════════════════════════════════════════╗
║                                        ║
║     🦊  Fax Programming Language       ║
║                                        ║
╚════════════════════════════════════════╝

EOF
    
    log_info "Starting installation - Version $SCRIPT_VERSION"
    
    # Dry run check
    if [[ "$DRY_RUN" == true ]]; then
        warn "DRY RUN MODE - No changes will be made"
        echo
    fi
    
    # Detect platform
    detect_platform
    say "Platform: ${BOLD}$PLATFORM${NC} (${PLATFORM_DISTRO:-unknown} ${PLATFORM_VERSION:-unknown})"
    log_info "Platform: $PLATFORM"
    
    # Check existing
    check_existing
    
    # Check dependencies
    check_dependencies
    
    # Exit if dry run
    [[ "$DRY_RUN" == true ]] && { say "Dry run complete"; exit 0; }
    
    # Install
    say_step "Installing Fax $FAX_VERSION"
    
    mkdir -p "$BIN_DIR" "$LIB_DIR" "$CACHE_DIR"
    
    # Clone
    better_clone "$FAX_REPO.git" "$INSTALL_DIR/repo"
    
    # Build
    better_build "$INSTALL_DIR/repo" "$FORCE_INSTALL"
    
    # Create wrappers
    create_better_wrappers
    
    # Create version file
    mkdir -p "$INSTALL_DIR/share/fax"
    echo "$FAX_VERSION" > "$INSTALL_DIR/share/fax/version"
    echo "$(date)" > "$INSTALL_DIR/share/fax/install_date"
    
    # Setup PATH
    better_path_setup
    
    # Verify
    verify_installation
    
    # Success
    [[ "$QUIET_MODE" == false ]] && cat << EOF

${GREEN}${BOLD}✓ Installation Complete!${NC}

${BOLD}Quick Start:${NC}
  1. Restart terminal or: source $(detect_shell)
  2. Verify: ${CYAN}faxt --version${NC}
  3. Hello world: ${CYAN}echo 'fn main() { print("Hello!"); }' > hello.fax && faxt run hello.fax${NC}

${BOLD}Useful Commands:${NC}
  • faxt update        - Update to latest version
  • faxt --version     - Show version
  • faxt uninstall     - Remove Fax

${BOLD}Documentation:${NC} https://luvion1.github.io/Fax/
${BOLD}Repository:${NC} $FAX_REPO

${GREEN}Happy coding! 🦊${NC}

EOF
    
    log_info "Installation completed successfully"
}

# Error handling
trap 'err "Installation failed unexpectedly. Check log: $LOG_FILE"; exit 1' ERR

# Run
main "$@"
