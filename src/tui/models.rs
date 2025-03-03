/// Input modes for the TUI
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    /// Normal mode - navigating and selecting workspaces
    Normal,
    
    /// Editing profile path
    ProfilePath,
    
    /// Selecting VSCode profile from known paths
    SelectProfile,
    
    /// Searching and filtering workspaces
    Searching,
    
    /// Confirming workspace deletion
    ConfirmDelete,
}

/// Simplified workspace info for the TUI
#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    /// Workspace ID
    #[allow(dead_code)]
    pub id: String,
    
    /// Workspace name (if available)
    pub name: Option<String>,
    
    /// Workspace path
    pub path: String,
    
    /// Whether the workspace exists on disk
    pub exists: bool,
    
    /// Workspace type (folder, file, git)
    pub workspace_type: String,
    
    /// Whether the workspace is remote
    pub is_remote: bool,
    
    /// Username for remote connections
    #[allow(dead_code)]
    pub remote_user: Option<String>,
    
    /// Port for remote connections
    #[allow(dead_code)]
    pub remote_port: Option<u16>,
    
    /// Tags associated with the workspace
    #[allow(dead_code)]
    pub tags: Vec<String>,
}

/// UI configuration settings
#[derive(Debug, Clone)]
pub struct UiConfig {
    /// Whether to use colors in the UI
    pub use_colors: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        // Check for NO_COLOR environment variable (a common standard)
        // https://no-color.org/
        let no_color = std::env::var("NO_COLOR").is_ok();
        
        Self {
            use_colors: !no_color,
        }
    }
} 