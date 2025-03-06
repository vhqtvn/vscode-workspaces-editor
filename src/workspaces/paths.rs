use anyhow::Result;
use directories::BaseDirs;
use home::home_dir;
use log::debug;

use crate::workspaces::error::WorkspaceError;

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
    // Return path as-is without any normalization
    uri_or_path.to_string()
}

/// Generate variations of a path to try for matching
pub fn generate_path_variations(path: &str) -> Vec<String> {
    let mut variations = Vec::new();

    // Original path
    variations.push(path.to_string());

    // Normalized path
    let normalized = normalize_path(path);
    if !variations.contains(&normalized) {
        variations.push(normalized.clone());
    }

    // With and without file:// prefix
    if path.starts_with("file://") {
        let without_prefix = path.replace("file://", "");
        if !variations.contains(&without_prefix) {
            variations.push(without_prefix);
        }
    } else {
        let with_prefix = format!("file://{}", path);
        if !variations.contains(&with_prefix) {
            variations.push(with_prefix);
        }
    }

    // Try both forward and backslash variations
    let with_forward_slashes = path.replace("\\", "/");
    if !variations.contains(&with_forward_slashes) {
        variations.push(with_forward_slashes);
    }

    let with_back_slashes = path.replace("/", "\\");
    if !variations.contains(&with_back_slashes) {
        variations.push(with_back_slashes);
    }

    // Try without drive letter on Windows (if present)
    if path.chars().nth(1) == Some(':') {
        let without_drive = &path[2..];
        if !variations.contains(&without_drive.to_string()) {
            variations.push(without_drive.to_string());
        }
    }

    // Remove trailing slashes
    let without_trailing_slash = path
        .trim_end_matches('/')
        .trim_end_matches('\\')
        .to_string();
    if !variations.contains(&without_trailing_slash) {
        variations.push(without_trailing_slash);
    }

    debug!(
        "Generated {} path variations for {}",
        variations.len(),
        path
    );
    variations
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
    let mut paths = Vec::new();

    // Try getting the default profile path
    if let Ok(default_path) = get_default_profile_path() {
        paths.push(default_path);
    }

    // Add potential alternative locations
    if let Some(home) = home_dir() {
        // Common Linux/Unix paths
        paths.push(home.join(".vscode").to_string_lossy().to_string());
        paths.push(home.join(".config/Code").to_string_lossy().to_string());
        paths.push(
            home.join(".config/Code - OSS")
                .to_string_lossy()
                .to_string(),
        );
        paths.push(home.join(".config/Cursor").to_string_lossy().to_string());

        // MacOS paths
        #[cfg(target_os = "macos")]
        {
            paths.push(
                home.join("Library/Application Support/Code")
                    .to_string_lossy()
                    .to_string(),
            );
            paths.push(
                home.join("Library/Application Support/Code - Insiders")
                    .to_string_lossy()
                    .to_string(),
            );
            paths.push(
                home.join("Library/Application Support/Cursor")
                    .to_string_lossy()
                    .to_string(),
            );
        }

        // Windows paths
        #[cfg(target_os = "windows")]
        {
            if let Some(base_dirs) = BaseDirs::new() {
                let data_dir = base_dirs.data_dir();
                paths.push(data_dir.join("Code").to_string_lossy().to_string());
                paths.push(
                    data_dir
                        .join("Code - Insiders")
                        .to_string_lossy()
                        .to_string(),
                );
                paths.push(data_dir.join("Cursor").to_string_lossy().to_string());
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
                                // Add VSCode paths for each Windows user
                                paths.push(
                                    path.join("AppData/Roaming/Code")
                                        .to_string_lossy()
                                        .to_string(),
                                );
                                paths.push(
                                    path.join("AppData/Roaming/Code - Insiders")
                                        .to_string_lossy()
                                        .to_string(),
                                );
                                paths.push(
                                    path.join("AppData/Local/Programs/Microsoft VS Code")
                                        .to_string_lossy()
                                        .to_string(),
                                );
                                paths.push(
                                    path.join("AppData/Local/Programs/Cursor")
                                        .to_string_lossy()
                                        .to_string(),
                                );
                                paths.push(path.join(".vscode").to_string_lossy().to_string());
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

    debug!("Found {} known VSCode paths", paths.len());
    paths
}
