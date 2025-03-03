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

# Check architecture
ARCH=$(uname -m)
if [[ "$ARCH" == "arm64" ]]; then
    echo -e "\033[31mError: ARM architecture (Apple Silicon) is not yet supported.\033[0m"
    echo "Currently only Intel (x86_64) Macs are supported. ARM support is planned for future releases."
    exit 1
fi

# Create necessary directories
mkdir -p "$INSTALL_DIR"

# Download the latest release
echo -e "\033[36mDownloading the latest CLI release...\033[0m"
LATEST_RELEASE_INFO=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest")
DOWNLOAD_URL=$(echo "$LATEST_RELEASE_INFO" | grep -o "browser_download_url.*cli.*macos.*x64.*tar.gz" | cut -d '"' -f 4 | head -n 1)

# If cli-specific package not found, try generic macOS package
if [ -z "$DOWNLOAD_URL" ]; then
    DOWNLOAD_URL=$(echo "$LATEST_RELEASE_INFO" | grep -o "browser_download_url.*macos.*x64.*tar.gz" | cut -d '"' -f 4 | head -n 1)
fi

if [ -z "$DOWNLOAD_URL" ]; then
    echo -e "\033[31mError: Could not find macOS CLI package in the latest release.\033[0m"
    echo "Please check the repository or try manual installation."
    exit 1
fi

TEMP_TAR="/tmp/vscode-workspaces-editor-cli.tar.gz"
echo -e "\033[36mDownloading from: $DOWNLOAD_URL\033[0m"
curl -L "$DOWNLOAD_URL" -o "$TEMP_TAR"

# Extract the archive
echo -e "\033[36mExtracting and installing...\033[0m"
TEMP_DIR="/tmp/vscode-workspaces-editor-cli"
mkdir -p "$TEMP_DIR"
tar -xzf "$TEMP_TAR" -C "$TEMP_DIR"

# Find and copy the CLI binary
CLI_BIN=$(find "$TEMP_DIR" -name "vscode-workspaces-editor" -type f | head -n 1)
if [ -z "$CLI_BIN" ]; then
    echo -e "\033[31mError: CLI binary not found in the downloaded package.\033[0m"
    exit 1
fi

cp "$CLI_BIN" "$INSTALL_DIR/vscode-workspaces-editor"
chmod +x "$INSTALL_DIR/vscode-workspaces-editor"

# Clean up
rm -rf "$TEMP_DIR" "$TEMP_TAR"

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