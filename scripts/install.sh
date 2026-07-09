#!/usr/bin/env bash

# Cypher CLI Installer for Unix-like systems (Linux & macOS)
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}[Cypher] CLI - Installer${NC}"
echo "----------------------------------------"

# 1. Check if Rust / Cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust / Cargo is not installed.${NC}"
    echo "Please install Rust by running:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo -e "${GREEN}[OK] Rust / Cargo detected.${NC}"

# 2. Compile Cypher CLI
INSTALL_DIR="$HOME/.cypher/bin"

if [ -f "Cargo.toml" ] && grep -q 'name = "cypher-cli"' Cargo.toml; then
    echo -e "${BLUE}Building directly from local source directory...${NC}"
    cargo build --release
    mkdir -p "$INSTALL_DIR"
    cp target/release/cypher-cli "$INSTALL_DIR/cypher"
else
    echo -e "${BLUE}Cloning Cypher repository temporarily to build...${NC}"
    TEMP_DIR=$(mktemp -d)
    git clone --depth 1 https://github.com/Ayan-Flash/Cypher.git "$TEMP_DIR"
    
    # Store current path
    CUR_DIR=$(pwd)
    
    cd "$TEMP_DIR"
    cargo build --release
    mkdir -p "$INSTALL_DIR"
    cp target/release/cypher-cli "$INSTALL_DIR/cypher"
    
    cd "$CUR_DIR"
    rm -rf "$TEMP_DIR"
fi

echo -e "${GREEN}[OK] Cypher CLI installed successfully as 'cypher'!${NC}"
echo "----------------------------------------"
echo -e "To run Cypher CLI, make sure ${BLUE}$INSTALL_DIR${NC} is in your PATH."
echo "You can add it by appending the following to your ~/.bashrc, ~/.zshrc, or ~/.profile:"
echo -e "\n  ${BLUE}export PATH=\"\$PATH:\$HOME/.cypher/bin\"${NC}\n"
echo "After adding it, reload your terminal or run: source ~/.bashrc (or ~/.zshrc)"
echo "Then check the version by running:"
echo -e "  ${GREEN}cypher --version${NC}"
