#!/usr/bin/env bash

# yame installer script
# Installs Rust/Cargo (if missing), fd, fzf, and yame.

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== yame Installation Script ===${NC}"

# 1. Detect Operating System
OS="$(uname -s)"
case "$OS" in
    Linux*)     OS_TYPE=Linux;;
    Darwin*)    OS_TYPE=Mac;;
    *)          OS_TYPE=Unknown;;
esac

if [ "$OS_TYPE" = "Unknown" ]; then
    echo -e "${RED}Error: Unsupported operating system ($OS). Please install manually.${NC}"
    exit 1
fi

# 2. Ensure Rust / Cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${BLUE}Cargo not found. Installing Rust toolchain...${NC}"
    if [ "$OS_TYPE" = "Mac" ] || [ "$OS_TYPE" = "Linux" ]; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # Load cargo environment
        source "$HOME/.cargo/env"
    fi
else
    echo -e "${GREEN}✓ Rust/Cargo is already installed.${NC}"
fi

# 3. Install soft dependencies (fd, fzf)
install_pkg() {
    local cmd="$1"
    if ! command -v "$cmd" &> /dev/null; then
        echo -e "${BLUE}Installing $cmd...${NC}"
        if [ "$OS_TYPE" = "Mac" ]; then
            if ! command -v brew &> /dev/null; then
                echo -e "${RED}Homebrew not found. Please install Homebrew or install $cmd manually.${NC}"
                return 1
            fi
            brew install "$cmd"
        elif [ "$OS_TYPE" = "Linux" ]; then
            if command -v apt-get &> /dev/null; then
                sudo apt-get update && sudo apt-get install -y "$cmd"
            elif command -v dnf &> /dev/null; then
                sudo dnf install -y "$cmd"
            elif command -v pacman &> /dev/null; then
                sudo pacman -S --noconfirm "$cmd"
            else
                echo -e "${RED}Unsupported package manager. Please install $cmd manually.${NC}"
                return 1
            fi
        fi
    else
        echo -e "${GREEN}✓ $cmd is already installed.${NC}"
    fi
}

install_pkg "fzf" || echo "Skipping fzf installation."
install_pkg "fd" || echo "Skipping fd installation."

# 4. Install yame
echo -e "${BLUE}Installing yame...${NC}"
cargo install yame

# 5. Output completion info
echo -e "\n${GREEN}=== Installation Complete ===${NC}"
echo -e "You can now run: ${BLUE}yame <filename>${NC}"
echo -e "To configure shell integrations (fuzzy searching), add the following to your .bashrc or .zshrc:"
echo -e "  ${BLUE}eval \"\$(yame init)\"${NC}"
