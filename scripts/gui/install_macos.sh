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

# Check architecture
ARCH=$(uname -m)
if [[ "$ARCH" == "arm64" ]]; then
    echo -e "\033[31mError: ARM architecture (Apple Silicon) is not yet supported.\033[0m"
    echo "Currently only Intel (x86_64) Macs are supported. ARM support is planned for future releases."
    exit 1
fi

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
LATEST_RELEASE_URL=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | grep "browser_download_url.*\.dmg" | grep -i "macos" | cut -d '"' -f 4)

if [ -z "$LATEST_RELEASE_URL" ]; then
    echo -e "\033[31mError: Could not find macOS installer in the latest release.\033[0m"
    echo "Please check the repository or try manual installation."
    exit 1
fi

TEMP_DMG="/tmp/vscode-workspaces-editor-gui.dmg"
curl -L "$LATEST_RELEASE_URL" -o "$TEMP_DMG"

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