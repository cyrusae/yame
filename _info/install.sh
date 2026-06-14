#!/usr/bin/env bash
# yame installer
# Installs Rust/Cargo (if missing), yame, and optional companion tools.

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

info()    { echo -e "${BLUE}▸ ${NC}$*"; }
success() { echo -e "${GREEN}✓ ${NC}$*"; }
warn()    { echo -e "${YELLOW}! ${NC}$*"; }
error()   { echo -e "${RED}✗ ${NC}$*" >&2; }
header()  { echo -e "\n${BOLD}$*${NC}"; }

ask() {
    # ask <prompt> — prints prompt, reads y/n, returns 0 for yes
    local prompt="$1"
    local answer
    read -r -p "$(echo -e "${BOLD}${prompt}${NC} [y/N] ")" answer
    [[ "$answer" =~ ^[Yy]$ ]]
}

# ---------------------------------------------------------------------------
# 1. OS detection
# ---------------------------------------------------------------------------

header "=== yame installer ==="

OS="$(uname -s 2>/dev/null || echo unknown)"
case "$OS" in
    Linux*)  OS_TYPE=Linux ;;
    Darwin*) OS_TYPE=Mac ;;
    *)
        error "Unsupported OS: $OS"
        error "Please follow the manual instructions in _info/SETUP.md"
        exit 1
        ;;
esac

# Detect Linux package manager
PKG_MGR=""
if [ "$OS_TYPE" = "Linux" ]; then
    if   command -v apt-get &>/dev/null; then PKG_MGR=apt
    elif command -v pacman  &>/dev/null; then PKG_MGR=pacman
    elif command -v dnf     &>/dev/null; then PKG_MGR=dnf
    elif command -v brew    &>/dev/null; then PKG_MGR=brew   # Linuxbrew
    fi
elif [ "$OS_TYPE" = "Mac" ]; then
    if command -v brew &>/dev/null; then PKG_MGR=brew; fi
fi

info "Detected: $OS_TYPE${PKG_MGR:+ (${PKG_MGR})}"

# ---------------------------------------------------------------------------
# Helper: install a package by name, handling Debian quirks
# ---------------------------------------------------------------------------

install_package() {
    local name="$1"         # display name
    local pkg_apt="${2:-$1}"
    local pkg_brew="${3:-$1}"
    local pkg_pacman="${4:-$1}"
    local pkg_dnf="${5:-$1}"

    case "$PKG_MGR" in
        apt)    sudo apt-get install -y "$pkg_apt" ;;
        brew)   brew install "$pkg_brew" ;;
        pacman) sudo pacman -S --noconfirm "$pkg_pacman" ;;
        dnf)    sudo dnf install -y "$pkg_dnf" ;;
        *)
            warn "No supported package manager found — please install $name manually."
            return 1
            ;;
    esac
}

# ---------------------------------------------------------------------------
# 2. Rust / Cargo
# ---------------------------------------------------------------------------

header "Rust toolchain"

if command -v cargo &>/dev/null; then
    success "Cargo already installed ($(cargo --version 2>/dev/null))"
else
    info "Cargo not found. Installing Rust toolchain via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
    success "Rust installed"
fi

# ---------------------------------------------------------------------------
# 3. Optional tools
# ---------------------------------------------------------------------------

header "Optional tools"
echo "  fd  — fast file finder (used by the yame shell integration wrapper)"
echo "  fzf — fuzzy finder     (used by the yame shell integration wrapper + Ctrl+O)"
echo "  lf  — file manager     (Ctrl+O inside yame opens lf directly)"
echo ""

INSTALL_FD=false
INSTALL_FZF=false
INSTALL_LF=false

# fd — Debian installs as fd-find / fdfind; others use fd
if command -v fd &>/dev/null || command -v fdfind &>/dev/null; then
    success "fd already installed"
else
    ask "Install fd?" && INSTALL_FD=true
fi

if command -v fzf &>/dev/null; then
    success "fzf already installed"
else
    ask "Install fzf?" && INSTALL_FZF=true
fi

if command -v lf &>/dev/null; then
    success "lf already installed"
else
    ask "Install lf?" && INSTALL_LF=true
fi

# Run installations
if $INSTALL_FD; then
    info "Installing fd..."
    # Debian packages it as fd-find; binary is fdfind — yame init handles this
    install_package "fd" "fd-find" "fd" "fd" "fd-find" \
        && success "fd installed"
fi

if $INSTALL_FZF; then
    info "Installing fzf..."
    install_package "fzf" && success "fzf installed"
fi

if $INSTALL_LF; then
    info "Installing lf..."
    install_package "lf" && success "lf installed"
fi

# ---------------------------------------------------------------------------
# 4. Install yame
# ---------------------------------------------------------------------------

header "Installing yame"
info "Running: cargo install yame"
cargo install yame
success "yame installed ($(yame --version 2>/dev/null))"

# ---------------------------------------------------------------------------
# 5. Shell integration
# ---------------------------------------------------------------------------

header "Shell integration"
echo "  Adding 'eval \"\$(yame init)\"' to your shell startup file lets you run"
echo "  'yame <term>' to fuzzy-search files and 'yame' to fuzzy-find Markdown files."
echo "  Requires fd and fzf."
echo ""

SHELL_RC=""
DETECTED_SHELL=""

case "${SHELL:-}" in
    *zsh)  DETECTED_SHELL=zsh;  SHELL_RC="$HOME/.zshrc" ;;
    *bash) DETECTED_SHELL=bash; SHELL_RC="$HOME/.bashrc" ;;
    *) warn "Could not detect shell from \$SHELL=${SHELL:-unset}. Skipping." ;;
esac

if [ -n "$SHELL_RC" ]; then
    INTEGRATION_LINE='eval "$(yame init)"'
    if grep -qF "$INTEGRATION_LINE" "$SHELL_RC" 2>/dev/null; then
        success "Shell integration already present in $SHELL_RC"
    else
        if ask "Add shell integration to $SHELL_RC?"; then
            echo "" >> "$SHELL_RC"
            echo "# yame shell integration (fuzzy file finding)" >> "$SHELL_RC"
            echo "$INTEGRATION_LINE" >> "$SHELL_RC"
            success "Added to $SHELL_RC — restart your shell or run: source $SHELL_RC"
        fi
    fi
fi

# ---------------------------------------------------------------------------
# 6. Config template
# ---------------------------------------------------------------------------

header "Config file"
echo "  'yame write-config' writes a commented config to ~/.config/yame/config.toml"
echo "  so you can see all available options."
echo ""

CONFIG_PATH="${XDG_CONFIG_HOME:-$HOME/.config}/yame/config.toml"

if [ -f "$CONFIG_PATH" ]; then
    success "Config already exists at $CONFIG_PATH"
else
    if ask "Generate a default config file?"; then
        yame write-config
    fi
fi

# ---------------------------------------------------------------------------
# 7. lf setup
# ---------------------------------------------------------------------------

if command -v lf &>/dev/null; then
    header "lf configuration"
    echo "  Configure lf to open files with yame and use 'yame --preview' for all file previews."
    echo ""

    if ask "Set up lf config for yame?"; then
        LFRC="$HOME/.config/lf/lfrc"
        PREVIEW="$HOME/.config/lf/preview"
        mkdir -p "$HOME/.config/lf"

        OPENER_LINE='cmd open $yame "$f"'
        PREVIEWER_LINE='set previewer ~/.config/lf/preview'

        if ! grep -qF 'cmd open' "$LFRC" 2>/dev/null; then
            echo "$OPENER_LINE" >> "$LFRC"
            info "Added opener to $LFRC"
        else
            warn "Opener already configured in $LFRC — skipping"
        fi

        if ! grep -qF 'set previewer' "$LFRC" 2>/dev/null; then
            echo "$PREVIEWER_LINE" >> "$LFRC"
            info "Added previewer line to $LFRC"
        else
            warn "Previewer already configured in $LFRC — skipping"
        fi

        if [ ! -f "$PREVIEW" ]; then
            cat > "$PREVIEW" << 'PREVIEW_EOF'
#!/usr/bin/env bash
# lf previewer — yame --preview handles all file types
COLUMNS="$2" yame --preview "$1"
PREVIEW_EOF
            chmod +x "$PREVIEW"
            success "Created $PREVIEW"
        else
            warn "$PREVIEW already exists — skipping"
        fi

        success "lf configured. Press Ctrl+O inside yame to open lf."
    fi
fi

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------

header "Done!"
echo ""
echo -e "  Open a file:     ${BLUE}yame README.md${NC}"
echo -e "  Untitled buffer: ${BLUE}yame${NC}"
echo -e "  Help:            ${BLUE}yame --help${NC}"
echo -e "  Key reference:   Press ${BLUE}F1${NC} inside yame"
echo ""
