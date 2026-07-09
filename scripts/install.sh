#!/usr/bin/env bash

# Cypher CLI Installer for Unix-like systems (Linux & macOS)
set -e

RED='\033[0;31m'; GREEN='\033[0;32m'; BLUE='\033[0;34m'; YELLOW='\033[1;33m'; NC='\033[0m'

echo -e "${BLUE}[Cypher] CLI - Installer${NC}"
echo "----------------------------------------"

INSTALL_DIR="$HOME/.cypher/bin"

# 1. Check/install Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Rust not found. Installing via rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source "$HOME/.cargo/env"
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}Failed to install Rust.${NC}"; exit 1
    fi
fi

echo -e "${GREEN}[OK] Rust / Cargo detected.${NC}"

# 2. Build Cypher CLI
if [ -f "Cargo.toml" ] && grep -q 'name = "cypher-cli"' Cargo.toml 2>/dev/null; then
    echo -e "${BLUE}Building from local source...${NC}"
    cargo build --release
    mkdir -p "$INSTALL_DIR"
    cp target/release/cypher "$INSTALL_DIR/cypher"
else
    echo -e "${BLUE}Cloning Cypher repository...${NC}"
    TEMP_DIR=$(mktemp -d)
    git clone --depth 1 https://github.com/Ayan-Flash/Cypher.git "$TEMP_DIR"
    cd "$TEMP_DIR"
    cargo build --release
    mkdir -p "$INSTALL_DIR"
    cp target/release/cypher "$INSTALL_DIR/cypher"
    cd "$HOME"
    rm -rf "$TEMP_DIR"
fi

echo -e "${GREEN}[OK] Cypher CLI installed as 'cypher'!${NC}"

# 3. Add to PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    SHELL_CONFIG="$HOME/.$(basename $SHELL)rc"
    [ ! -f "$SHELL_CONFIG" ] && SHELL_CONFIG="$HOME/.profile"
    echo "" >> "$SHELL_CONFIG"
    echo 'export PATH="$PATH:'$INSTALL_DIR'"' >> "$SHELL_CONFIG"
    export PATH="$PATH:$INSTALL_DIR"
    echo -e "${GREEN}[OK] Added to PATH in $SHELL_CONFIG${NC}"
fi

echo ""
echo -e "Done! Run: ${GREEN}cypher --version${NC}"
