#!/bin/bash
# VSCode Workspaces Editor CLI installer script for Linux

set -e

# Configuration
APP_NAME="vscode-workspaces-editor"
VERSION="1.0.0"
GITHUB_REPO="vhqtvn/vscode-workspaces-editor"
INSTALL_DIR="$HOME/.local/bin"

# Output colors
GREEN='\033[0;32m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}VSCode Workspaces Editor CLI Installer for Linux${NC}"
echo "========================================="

# Create necessary directories
echo -e "${CYAN}Creating directories...${NC}"
mkdir -p "$INSTALL_DIR"

# Determine system architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)
        ARCH_NAME="x64"
        ;;
    *)
        echo -e "${RED}Unsupported architecture: $ARCH${NC}"
        echo "Currently only x86_64 is supported. ARM architecture support is coming in future releases."
        exit 1
        ;;
esac

# Download the latest release
echo -e "${CYAN}Downloading the latest CLI release...${NC}"
LATEST_RELEASE_INFO=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest")
DOWNLOAD_URL=$(echo "$LATEST_RELEASE_INFO" | grep -o "browser_download_url.*cli.*linux.*$ARCH_NAME.*tar.gz" | cut -d '"' -f 4 | head -n 1)

# If CLI-specific package not found, try generic Linux package
if [ -z "$DOWNLOAD_URL" ]; then
    DOWNLOAD_URL=$(echo "$LATEST_RELEASE_INFO" | grep -o "browser_download_url.*linux.*$ARCH_NAME.*tar.gz" | cut -d '"' -f 4 | head -n 1)
fi

if [ -z "$DOWNLOAD_URL" ]; then
    echo -e "${RED}Error: Could not find Linux CLI package in the latest release.${NC}"
    echo "Please check the repository or try manual installation."
    exit 1
fi

TEMP_TAR="/tmp/vscode-workspaces-editor-cli.tar.gz"
echo -e "${CYAN}Downloading from: $DOWNLOAD_URL${NC}"
curl -L "$DOWNLOAD_URL" -o "$TEMP_TAR"

# Extract the archive
echo -e "${CYAN}Extracting and installing...${NC}"
TEMP_DIR="/tmp/vscode-workspaces-editor-cli"
mkdir -p "$TEMP_DIR"
tar -xzf "$TEMP_TAR" -C "$TEMP_DIR"

# Find and copy the CLI binary
CLI_BIN=$(find "$TEMP_DIR" -name "vscode-workspaces-editor" -type f | head -n 1)
if [ -z "$CLI_BIN" ]; then
    echo -e "${RED}Error: CLI binary not found in the downloaded package.${NC}"
    exit 1
fi

cp "$CLI_BIN" "$INSTALL_DIR/vscode-workspaces-editor"
chmod +x "$INSTALL_DIR/vscode-workspaces-editor"

# Clean up
rm -rf "$TEMP_DIR" "$TEMP_TAR"

echo -e "${GREEN}CLI Installation completed successfully!${NC}"
echo "You can now run 'vscode-workspaces-editor' from the terminal."

# Check if $INSTALL_DIR is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${CYAN}NOTE: $INSTALL_DIR is not in your PATH.${NC}"
    echo "You may need to add it to your PATH in your shell profile:"
    echo "export PATH=\"\$PATH:$INSTALL_DIR\""
fi 