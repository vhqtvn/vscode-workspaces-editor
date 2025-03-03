#!/bin/bash
# VSCode Workspaces Editor GUI installer script for Linux

set -e

# Configuration
APP_NAME="vscode-workspaces-editor-gui"
VERSION="1.0.0"
GITHUB_REPO="vhqtvn/vscode-workspaces-editor"
INSTALL_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/vscode-workspaces-editor"
DESKTOP_FILE_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/128x128/apps"

# Output colors
GREEN='\033[0;32m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}VSCode Workspaces Editor GUI Installer for Linux${NC}"
echo "========================================="

# Create necessary directories
echo -e "${CYAN}Creating directories...${NC}"
mkdir -p "$INSTALL_DIR"
mkdir -p "$APP_DIR"
mkdir -p "$DESKTOP_FILE_DIR"
mkdir -p "$ICON_DIR"

# Determine system architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)
        ARCH_NAME="x64"
        ;;
    aarch64|arm64)
        ARCH_NAME="arm64"
        ;;
    armv7*|armhf)
        ARCH_NAME="arm"
        ;;
    *)
        echo -e "${RED}Unsupported architecture: $ARCH${NC}"
        echo "Currently only x86_64, arm64, and armv7 are supported."
        exit 1
        ;;
esac

# Download the latest release
echo -e "${CYAN}Downloading the latest GUI release...${NC}"
LATEST_RELEASE_INFO=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest")

# Try to find an AppImage first
DOWNLOAD_URL=$(echo "$LATEST_RELEASE_INFO" | grep -o "browser_download_url.*vscode-workspaces-editor-gui-linux-$ARCH_NAME.AppImage[^\"]*" | cut -d '"' -f 4 | head -n 1)

# If no AppImage, try .deb
if [ -z "$DOWNLOAD_URL" ] && [ "$ARCH_NAME" != "arm" ]; then
    DOWNLOAD_URL=$(echo "$LATEST_RELEASE_INFO" | grep -o "browser_download_url.*vscode-workspaces-editor-gui-linux-$ARCH_NAME.deb[^\"]*" | cut -d '"' -f 4 | head -n 1)
    USE_DEB=true
fi

if [ -z "$DOWNLOAD_URL" ]; then
    echo -e "${RED}Error: Could not find Linux GUI package for $ARCH_NAME in the latest release.${NC}"
    echo "Please check the repository or try manual installation."
    exit 1
fi

if [ "$USE_DEB" = true ]; then
    # Handle .deb installation
    TEMP_DEB="/tmp/vscode-workspaces-editor-gui.deb"
    echo -e "${CYAN}Downloading from: $DOWNLOAD_URL${NC}"
    curl -L "$DOWNLOAD_URL" -o "$TEMP_DEB"
    
    # Install .deb
    echo -e "${CYAN}Installing .deb package...${NC}"
    if command -v apt &> /dev/null; then
        sudo apt install -y "$TEMP_DEB"
    elif command -v dpkg &> /dev/null; then
        sudo dpkg -i "$TEMP_DEB"
        sudo apt-get install -f -y
    else
        echo -e "${RED}Error: Neither apt nor dpkg found. Cannot install .deb package.${NC}"
        echo "Please install manually from $TEMP_DEB"
        exit 1
    fi
    
    # Clean up
    rm -f "$TEMP_DEB"
else
    # Handle AppImage installation
    TEMP_APPIMAGE="/tmp/vscode-workspaces-editor-gui.AppImage"
    echo -e "${CYAN}Downloading from: $DOWNLOAD_URL${NC}"
    curl -L "$DOWNLOAD_URL" -o "$TEMP_APPIMAGE"
    chmod +x "$TEMP_APPIMAGE"
    
    # Extract AppImage if possible
    echo -e "${CYAN}Extracting and installing...${NC}"
    cp "$TEMP_APPIMAGE" "$APP_DIR/vscode-workspaces-editor-gui.AppImage"
    chmod +x "$APP_DIR/vscode-workspaces-editor-gui.AppImage"
    
    # Create symlinks for GUI access
    ln -sf "$APP_DIR/vscode-workspaces-editor-gui.AppImage" "$INSTALL_DIR/vscode-workspaces-editor-gui"
    
    # Extract icon from AppImage if possible
    "$APP_DIR/vscode-workspaces-editor-gui.AppImage" --appimage-extract *.png 2>/dev/null || true
    if [ -d "squashfs-root" ]; then
        ICON_FILE=$(find squashfs-root -name "*.png" | head -n 1)
        if [ -n "$ICON_FILE" ]; then
            cp "$ICON_FILE" "$ICON_DIR/vscode-workspaces-editor.png"
        fi
        rm -rf squashfs-root
    fi
    
    # Create desktop entry
    echo -e "${CYAN}Creating desktop entry...${NC}"
    cat > "$DESKTOP_FILE_DIR/vscode-workspaces-editor.desktop" << EOF
[Desktop Entry]
Name=VSCode Workspaces Editor
Comment=Manage Visual Studio Code workspaces
Exec=$APP_DIR/vscode-workspaces-editor-gui.AppImage
Icon=vscode-workspaces-editor
Terminal=false
Type=Application
Categories=Development;Utility;
StartupWMClass=vscode-workspaces-editor-gui
EOF
    
    # Clean up
    rm -f "$TEMP_APPIMAGE"
fi

echo -e "${GREEN}GUI Installation completed successfully!${NC}"
echo "You can find 'VSCode Workspaces Editor' in your application menu."
echo "You can also run 'vscode-workspaces-editor-gui' from the command line."

# Check if $INSTALL_DIR is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${CYAN}NOTE: $INSTALL_DIR is not in your PATH.${NC}"
    echo "You may need to add it to your PATH in your shell profile:"
    echo "export PATH=\"\$PATH:$INSTALL_DIR\""
fi 