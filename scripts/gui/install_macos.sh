#!/bin/bash
# VSCode Workspaces Editor GUI installer script for macOS

set -e

# Configuration
APP_NAME="vscode-workspaces-editor-gui"
VERSION="1.0.0"
GITHUB_REPO="vhqtvn/vscode-workspaces-editor"
INSTALL_DIR="/Applications"

echo -e "\033[36mVSCode Workspaces Editor GUI Installer for macOS\033[0m"
echo "========================================"

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
        ARCH_NAME="x64"
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

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo -e "\033[33mHomebrew not found. It's recommended for dependency management.\033[0m"
    echo "Would you like to install Homebrew? (y/n)"
    read install_brew
    
    if [[ "$install_brew" == "y" || "$install_brew" == "Y" ]]; then
        echo -e "\033[36mInstalling Homebrew...\033[0m"
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi
fi

# Download the latest release
echo -e "\033[36mDownloading the latest GUI release...\033[0m"
LATEST_RELEASE_INFO=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest")
DOWNLOAD_URL=$(echo "$LATEST_RELEASE_INFO" | jq -r ".assets[] | select(.name | contains(\"vscode-workspaces-editor-gui-macos-$ARCH_NAME\") and contains(\".dmg\")) | .browser_download_url")

if [ -z "$DOWNLOAD_URL" ]; then
    echo -e "\033[31mError: Could not find macOS DMG installer for $ARCH_NAME in the latest release.\033[0m"
    echo "Please check the repository or try manual installation."
    exit 1
fi

TEMP_DMG="/tmp/vscode-workspaces-editor-gui.dmg"
echo -e "\033[36mDownloading from: $DOWNLOAD_URL\033[0m"
curl -L "$DOWNLOAD_URL" -o "$TEMP_DMG"

# Mount the DMG
echo -e "\033[36mMounting disk image...\033[0m"
MOUNT_POINT=$(hdiutil attach "$TEMP_DMG" -nobrowse -noautoopen | tail -n 1 | awk '{print $NF}')

# Copy the application to Applications directory
echo -e "\033[36mInstalling application to $INSTALL_DIR...\033[0m"
cp -R "$MOUNT_POINT"/*.app "$INSTALL_DIR"

# Unmount the DMG
echo -e "\033[36mCleaning up...\033[0m"
hdiutil detach "$MOUNT_POINT" -quiet

# Remove the downloaded DMG
rm "$TEMP_DMG"

# Create symlink for GUI access
echo -e "\033[36mCreating command-line shortcut for GUI...\033[0m"
if [ -d "/usr/local/bin" ]; then
    CLI_DIR="/usr/local/bin"
elif [ -d "$HOME/.local/bin" ]; then
    CLI_DIR="$HOME/.local/bin"
else
    mkdir -p "$HOME/.local/bin"
    CLI_DIR="$HOME/.local/bin"
    echo "export PATH=\$PATH:$HOME/.local/bin" >> "$HOME/.bash_profile"
    echo "export PATH=\$PATH:$HOME/.local/bin" >> "$HOME/.zshrc"
fi

ln -sf "$INSTALL_DIR/VSCode Workspaces Editor.app/Contents/MacOS/vscode-workspaces-editor-gui" "$CLI_DIR/vscode-workspaces-editor-gui"

echo -e "\033[32mInstallation completed successfully!\033[0m"
echo "You can find 'VSCode Workspaces Editor' in your Applications folder."
echo "You can also run 'vscode-workspaces-editor-gui' from the terminal to launch the GUI." 