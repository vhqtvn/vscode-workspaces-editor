#!/bin/bash
# VSCode Workspaces Editor CLI installer script for macOS

set -e

# Configuration
APP_NAME="vscode-workspaces-editor"
VERSION="1.0.0"
GITHUB_REPO="vhqtvn/vscode-workspaces-editor"
INSTALL_DIR="$HOME/.local/bin"

echo -e "\033[36mVSCode Workspaces Editor CLI Installer for macOS\033[0m"
echo "========================================"

# Create necessary directories
mkdir -p "$INSTALL_DIR"

# Check for jq
if ! command -v jq &> /dev/null; then
    echo -e "\033[31mError: jq is required but not installed.\033[0m"
    echo "Please install jq first using:"
    echo "  brew install jq"
    echo "Or download from: https://stedolan.github.io/jq/download/"
    exit 1
fi

# Determine system architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)
        ARCH_NAME="amd64"
        ;;
    arm64)
        ARCH_NAME="arm64"
        ;;
    *)
        echo -e "\033[31mError: Unsupported architecture: $ARCH\033[0m"
        echo "Currently only x86_64 and arm64 are supported."
        exit 1
        ;;
esac

# Download the latest release
echo -e "\033[36mDownloading the latest CLI release...\033[0m"
LATEST_RELEASE_INFO=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest")
DOWNLOAD_URL=$(echo "$LATEST_RELEASE_INFO" | jq -r ".assets[] | select(.name | contains(\"vscode-workspaces-editor-macos-$ARCH_NAME\")) | .browser_download_url")

if [ -z "$DOWNLOAD_URL" ]; then
    echo -e "\033[31mError: Could not find macOS CLI binary for $ARCH_NAME in the latest release.\033[0m"
    echo "Please check the repository or try manual installation."
    exit 1
fi

TEMP_BIN="/tmp/vscode-workspaces-editor"
echo -e "\033[36mDownloading from: $DOWNLOAD_URL\033[0m"
curl -L "$DOWNLOAD_URL" -o "$TEMP_BIN"
chmod +x "$TEMP_BIN"

# Install the binary
echo -e "\033[36mInstalling CLI binary...\033[0m"
cp "$TEMP_BIN" "$INSTALL_DIR/$APP_NAME"
chmod +x "$INSTALL_DIR/$APP_NAME"

# Clean up
rm -f "$TEMP_BIN"

# Add to PATH if needed
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "\033[36mAdding $INSTALL_DIR to PATH...\033[0m"
    
    # Determine shell and update appropriate profile
    if [[ "$SHELL" == *"zsh"* ]]; then
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$HOME/.zshrc"
        echo -e "\033[33mNOTE: Added $INSTALL_DIR to PATH in .zshrc\033[0m"
        echo "To use immediately, run: export PATH=\"\$PATH:$INSTALL_DIR\""
    elif [[ "$SHELL" == *"bash"* ]]; then
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$HOME/.bash_profile"
        echo -e "\033[33mNOTE: Added $INSTALL_DIR to PATH in .bash_profile\033[0m"
        echo "To use immediately, run: export PATH=\"\$PATH:$INSTALL_DIR\""
    else
        echo -e "\033[33mNOTE: Please add $INSTALL_DIR to your PATH manually.\033[0m"
    fi
fi

echo -e "\033[32mCLI Installation completed successfully!\033[0m"
echo "You can now run 'vscode-workspaces-editor' from the terminal." 