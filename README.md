# VSCode Workspaces Editor

A powerful desktop application and terminal interface for managing Visual Studio Code workspaces. This tool provides an intuitive way to manage your VSCode workspace files.

## Features

- Desktop application with modern UI (powered by Tauri)
- Terminal interface for command-line operations
- Create and edit VSCode workspace files
- Manage multiple workspaces efficiently
- Cross-platform support

## Installation

### Automatic Installation

We provide platform-specific installation scripts that automatically download and install the latest release.

> **Note:** Currently, only x86_64 (64-bit) architecture is supported. ARM architecture support is planned for future releases.

#### GUI Installation (Desktop Application)

##### Windows

```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/vhqtvn/vscode-workspaces-editor/main/scripts/gui/install_windows.ps1'))
```

##### macOS

```bash
curl -fsSL https://raw.githubusercontent.com/vhqtvn/vscode-workspaces-editor/main/scripts/gui/install_macos.sh | bash
```

##### Linux

```bash
curl -fsSL https://raw.githubusercontent.com/vhqtvn/vscode-workspaces-editor/main/scripts/gui/install_linux.sh | bash
```

#### CLI Installation (Terminal Interface)

##### Windows

```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/vhqtvn/vscode-workspaces-editor/main/scripts/cli/install_windows.ps1'))
```

##### macOS

```bash
curl -fsSL https://raw.githubusercontent.com/vhqtvn/vscode-workspaces-editor/main/scripts/cli/install_macos.sh | bash
```

##### Linux

```bash
curl -fsSL https://raw.githubusercontent.com/vhqtvn/vscode-workspaces-editor/main/scripts/cli/install_linux.sh | bash
```

### Manual Installation

#### Download Binaries

You can download the latest binaries from the [Releases page](https://github.com/vhqtvn/vscode-workspaces-editor/releases).

Available packages:

**GUI (Desktop Application):**
- Windows: `.msi` installer or `.zip` archive
- macOS: `.dmg` disk image
- Linux: `.AppImage` or `.deb`/`.rpm` packages

**CLI (Terminal Interface):**
- Windows: `.zip` archive containing the CLI executable
- macOS: `.tar.gz` archive containing the CLI binary
- Linux: `.tar.gz` archive containing the CLI binary

#### Building from Source

##### Prerequisites

- Rust (latest stable version)
- Node.js (for UI development)
- Visual Studio Code

##### Build Steps

1. Clone the repository:
```bash
git clone https://github.com/vhqtvn/vscode-workspace-editor.git
cd vscode-workspace-editor
```

2. Build the project:
```bash
cargo build --release
```

The built binary will be available in `target/release/vscode-workspaces-editor`

## Usage

### Terminal Interface

Run the application in terminal mode:

```bash
vscode-workspaces-editor
```

### Desktop Application

Launch the desktop application:

```bash
vscode-workspaces-editor-gui
```

## Project Structure

- `src/` - Core Rust application code
- `src-tauri/` - Tauri application configuration and native code
- `src-ui/` - Web-based user interface code
- `scripts/` - Installation scripts for different platforms
  - `scripts/gui/` - GUI-specific installation scripts
  - `scripts/cli/` - CLI-specific installation scripts

## Dependencies

Major dependencies include:
- Tauri 2.0.0 - Desktop application framework
- Ratatui 0.24 - Terminal user interface
- SQLite (via rusqlite 0.29) - Data storage
- Tokio 1.32 - Async runtime
- Various utility crates for enhanced functionality

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License

## Author

[vhqtvn](https://github.com/vhqtvn)
