use anyhow::Result;
use directories::BaseDirs;
use home::home_dir;
use log::debug;

use crate::workspaces::error::WorkspaceError;
use crate::workspaces::zed::ZED_PROFILE_NAME;

/// Get the default VSCode profile path for the current platform
pub fn get_default_profile_path() -> Result<String> {
    if let Some(base_dirs) = BaseDirs::new() {
        #[allow(unused_variables)]
        let config_dir = base_dirs.config_dir();

        #[cfg(target_os = "macos")]
        let path = config_dir
            .parent()
            .unwrap_or(config_dir)
            .join("Application Support/Code");

        #[cfg(target_os = "windows")]
        let path = base_dirs.data_dir().join("Code");

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let path = config_dir.join("Code");

        return Ok(path.to_string_lossy().to_string());
    }

    // Fallback to $HOME/.config/Code for Linux
    let home = home_dir().ok_or(WorkspaceError::HomeDir)?;
    Ok(home.join(".config/Code").to_string_lossy().to_string())
}

/// Expand ~ in path to home directory
pub fn expand_tilde(path: &str) -> Result<String> {
    if let Some(stripped) = path.strip_prefix("~") {
        let home = home_dir().ok_or(WorkspaceError::HomeDir)?;
        let path_without_leading_slash = stripped.trim_start_matches('/');

        Ok(home
            .join(path_without_leading_slash)
            .to_string_lossy()
            .to_string())
    } else {
        Ok(path.to_string())
    }
}

/// Normalize a path or URI to a consistent format
pub fn normalize_path(uri_or_path: &str) -> String {
    debug!("Normalizing path: {}", uri_or_path);
    
    // First decode any URL encoding
    let decoded = match urlencoding::decode(uri_or_path) {
        Ok(decoded) => decoded.into_owned(),
        Err(_) => uri_or_path.to_string(),
    };
    
    // Handle file:// and vscode-remote:// prefixes
    let path = if decoded.starts_with("vscode-remote://") {
        // Keep remote paths as-is to maintain uniqueness
        decoded
    } else if decoded.starts_with("file://") {
        // Remove file:// prefix and normalize
        decoded.replace("file://", "")
    } else {
        decoded
    };
    
    // Remove any trailing slashes
    let clean_path = path.trim_end_matches('/').trim_end_matches('\\');
    
    // Normalize path separators to forward slashes
    let normalized = clean_path.replace('\\', "/");
    
    debug!("Normalized result: {}", normalized);
    normalized
}

/// Check if we're running inside WSL
fn is_wsl() -> bool {
    if let Ok(release) = std::fs::read_to_string("/proc/version") {
        return release.to_lowercase().contains("microsoft")
            || release.to_lowercase().contains("wsl");
    }
    false
}

/// Get all possible known VSCode configuration paths for the current system
pub fn get_known_vscode_paths() -> Vec<String> {
    let code_compatible_programs = vec![
        "Code",
        "Code - Insiders",
        "Cursor",
        "Antigravity",
        "Kiro",
    ];
    let mut paths = Vec::new();

    // Try getting the default profile path
    if let Ok(default_path) = get_default_profile_path() {
        paths.push(default_path);
    }

    // Add potential alternative locations
    if let Some(home) = home_dir() {
        // Common Linux/Unix paths
        paths.push(home.join(".vscode").to_string_lossy().to_string());
        paths.extend(code_compatible_programs.iter().map(
            |p| home.join(".config").join(p).to_string_lossy().to_string()
        ));

        // MacOS paths
        #[cfg(target_os = "macos")]
        {
            paths.extend(code_compatible_programs.iter().map(
                |p| home.join("Library/Application Support").join(p).to_string_lossy().to_string()
            ));
        }

        // Windows paths
        #[cfg(target_os = "windows")]
        {
            if let Some(base_dirs) = BaseDirs::new() {
                let data_dir = base_dirs.data_dir();
                paths.extend(code_compatible_programs.iter().map(
                    |p| data_dir.join(p).to_string_lossy().to_string()
                ));
            }
        }

        // WSL-specific paths
        if is_wsl() {
            // Try to find Windows user directories through WSL mount
            if let Ok(entries) = std::fs::read_dir("/mnt/c/Users") {
                for entry in entries.flatten() {
                    if let Ok(path) = entry.path().canonicalize() {
                        if let Ok(metadata) = path.metadata() {
                            if metadata.is_dir() {
                                paths.push(path.join(".vscode").to_string_lossy().to_string());
                                paths.extend(
                                    code_compatible_programs.iter().map(
                                        |p| path.join("AppData/Roaming").join(p)
                                            .to_string_lossy()
                                            .to_string()
                                    )
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Remove duplicates and normalize all paths
    paths = paths
        .into_iter()
        .map(|p| normalize_path(&p))
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();

    paths = paths
        .into_iter()
        .filter(|p| std::path::Path::new(p).is_dir())
        .collect::<Vec<_>>();

    // Add fake profiles that don't correspond to actual directories
    paths.push(ZED_PROFILE_NAME.to_string());

    debug!("Found {} known VSCode paths", paths.len());
    paths
}
