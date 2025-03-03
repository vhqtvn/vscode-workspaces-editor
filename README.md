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

> **Note:** ARM architecture (ARM64/aarch64) is fully supported on macOS and Windows. The installer scripts will detect your system architecture and download the appropriate version. Linux is currently supported only on x86_64 architecture.

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
- Windows: `.msi` installer (x64 and ARM64)
- macOS: `.dmg` disk image (x64 and ARM64)
- Linux: `.AppImage` or `.deb`/`.rpm` packages (x64 only)

**CLI (Terminal Interface):**
- Windows: executable for x64 and ARM64
- macOS: binary for x64 and ARM64
- Linux: binary for x64 only

#### Building from Source

##### Prerequisites

- Rust (latest stable version)
- Node.js (for UI development)
- Visual Studio Code

##### Build Steps

1. Clone the repository:
```bash
git clone https://github.com/vhqtvn/vscode-workspaces-editor.git
cd vscode-workspaces-editor
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
