# VSCode Workspaces Editor

A powerful desktop application and terminal interface for managing Visual Studio Code workspaces. This tool provides an intuitive way to manage your VSCode workspace files.

## Features

- Desktop application with modern UI (powered by Tauri)
- Terminal interface for command-line operations
- Create and edit VSCode workspace files
- Manage multiple workspaces efficiently
- Cross-platform support

## Installation

### Prerequisites

- Rust (latest stable version)
- Node.js (for UI development)
- Visual Studio Code

### Building from Source

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
