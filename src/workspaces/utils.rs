use log::info;
use std::path::Path;
use anyhow::Result;
use crate::workspaces::models::Workspace;
use crate::workspaces::parser::WorkspaceType;
use log::debug;

/// Check if a directory exists
#[allow(dead_code)]
pub fn directory_exists(path: &str) -> bool {
    let path = Path::new(path);
    if path.exists() && path.is_dir() {
        info!("Directory exists: {}", path.display());
        true
    } else {
        if !path.exists() {
            info!("Path does not exist: {}", path.display());
        } else {
            info!("Path is not a directory: {}", path.display());
        }
        false
    }
}

/// Check if a workspace path exists (handles both local and remote paths)
pub fn workspace_exists(workspace: &Workspace) -> bool {
    // Parse workspace path if not already parsed
    // Using clone to avoid mutable borrow
    let mut workspace_clone = workspace.clone();
    let parsed_info = workspace_clone.parse_path();
    
    // Check if this is a remote workspace
    let is_remote = if let Some(info) = parsed_info {
        info.remote_authority.is_some()
    } else {
        false
    };
    
    if is_remote {
        // For remote workspaces, we can't check directly
        // TODO: Implement actual remote path checking in the future
        debug!("Remote workspace existence check not implemented: {}", workspace.path);
        return true; // Assume remote paths exist
    }
    
    // For local paths, check if the file or directory exists
    let path = Path::new(&workspace.path);
    let path_str = path.to_string_lossy();
    
    // Remove file:// prefix if present
    let clean_path = if path_str.starts_with("file://") {
        path_str.replace("file://", "")
    } else {
        path_str.to_string()
    };
    
    // Check if this is a workspace or a folder/file
    if clean_path.ends_with(".code-workspace") {
        let workspace_path = Path::new(&clean_path);
        if workspace_path.exists() && workspace_path.is_file() {
            debug!("Workspace file exists: {}", clean_path);
            true
        } else {
            debug!("Workspace file does not exist: {}", clean_path);
            false
        }
    } else {
        let dir_path = Path::new(&clean_path);
        if dir_path.exists() {
            if dir_path.is_dir() {
                debug!("Directory exists: {}", clean_path);
                true
            } else {
                debug!("Path exists but is not a directory: {}", clean_path);
                true // Consider files as valid targets too
            }
        } else {
            debug!("Path does not exist: {}", clean_path);
            false
        }
    }
}

/// Check if VSCode is installed and available
#[allow(dead_code)]
pub fn is_vscode_available() -> bool {
    match std::process::Command::new("code")
        .arg("--version")
        .output() {
        Ok(_) => true,
        Err(e) => {
            info!("VSCode is not available: {}", e);
            false
        }
    }
}

/// Process workspaces to add parsed information
pub fn process_workspaces(workspaces: &mut [Workspace]) -> Result<()> {
    for workspace in workspaces.iter_mut() {
        // Parse and add workspace path information
        let _ = workspace.parse_path();
    }
    Ok(())
}

/// Extract the folder basename from a path
/// Handles different types of paths including remote and container paths
pub fn extract_folder_basename(path: &str) -> String {
    // If it's a file:// URI, remove the prefix
    let clean_path = if path.starts_with("file://") {
        path.replace("file://", "")
    } else {
        path.to_string()
    };
    
    // For local paths, just extract the basename
    if !path.starts_with("vscode-remote://") {
        return Path::new(&clean_path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".to_string());
    }
    
    // For remote paths, we need to parse the path component
    if let Ok(info) = crate::workspaces::parser::parse_workspace_path(path) {
        // Get the local path from the parsed information
        Path::new(&info.path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".to_string())
    } else {
        // Fallback
        "unnamed".to_string()
    }
}

/// Filter workspaces by different criteria
#[allow(dead_code)]
pub fn filter_workspaces<'a>(workspaces: &'a mut [Workspace], query: &str) -> Vec<&'a Workspace> {
    let query = query.trim().to_lowercase();
    
    // Pre-parse all workspaces before filtering
    for workspace in workspaces.iter_mut() {
        let _ = workspace.parse_path();
    }
    
    // If query is empty, return all workspaces
    if query.is_empty() {
        return workspaces.iter().collect();
    }
    
    // Parse query parts
    let query_parts: Vec<&str> = query.split(' ').collect();
    
    // Process filter parts like :remote:, :type:, etc.
    let mut remote_filter: Option<Vec<&str>> = None;
    let mut type_filter: Option<Vec<&str>> = None;
    let mut path_filter: Option<Vec<&str>> = None;
    let mut tag_filter: Option<Vec<&str>> = None;
    let mut existing_filter: Option<bool> = None;
    let mut text_query = String::new();
    
    for part in query_parts {
        if let Some(stripped) = part.strip_prefix(":remote:") {
            remote_filter = Some(stripped.split(',').collect());
        } else if let Some(stripped) = part.strip_prefix(":type:") {
            type_filter = Some(stripped.split(',').collect());
        } else if let Some(stripped) = part.strip_prefix(":path:") {
            path_filter = Some(stripped.split(',').collect());
        } else if let Some(stripped) = part.strip_prefix(":tag:") {
            tag_filter = Some(stripped.split(',').collect());
        } else if let Some(stripped) = part.strip_prefix(":tags:") {
            tag_filter = Some(stripped.split(',').collect());
        } else if let Some(stripped) = part.strip_prefix(":existing:") {
            let value = stripped;
            if value == "true" || value == "yes" || value == "1" {
                existing_filter = Some(true);
            } else if value == "false" || value == "no" || value == "0" {
                existing_filter = Some(false);
            }
        } else if !part.is_empty() {
            if !text_query.is_empty() {
                text_query.push(' ');
            }
            text_query.push_str(part);
        }
    }
    
    debug!("Filtering workspaces with: text='{}', remote={:?}, type={:?}, path={:?}, tag={:?}, existing={:?}",
        text_query, remote_filter, type_filter, path_filter, tag_filter, existing_filter);
    
    workspaces.iter()
        .filter(|ws| {
            // Check text search (path, name, label)
            if !text_query.is_empty() {
                let path_match = ws.path.to_lowercase().contains(&text_query);
                let name_match = ws.name.as_ref()
                    .map(|n| n.to_lowercase().contains(&text_query))
                    .unwrap_or(false);
                let label = if let Some(name) = &ws.name {
                    if !name.is_empty() {
                        name.clone()
                    } else {
                        ws.path.clone()
                    }
                } else {
                    ws.path.clone()
                };
                let label_match = label.to_lowercase().contains(&text_query);
                
                if !path_match && !name_match && !label_match {
                    return false;
                }
            }
            
            // Check remote filter
            if let Some(remote_values) = &remote_filter {
                if let Some(info) = &ws.parsed_info {
                    if let Some(remote) = &info.remote_host {
                        if !remote_values.iter().any(|&val| remote.to_lowercase().contains(val)) {
                            return false;
                        }
                    } else {
                        // No remote host, but filter requires one
                        return false;
                    }
                } else {
                    return false;
                }
            }
            
            // Check workspace type filter
            if let Some(type_values) = &type_filter {
                let ws_type = match &ws.parsed_info {
                    Some(info) => match info.workspace_type {
                        WorkspaceType::Folder => "folder",
                        WorkspaceType::File => "file",
                        WorkspaceType::Workspace => "workspace",
                    },
                    None => "folder", // default to folder if parsing fails
                };
                
                if !type_values.iter().any(|&val| ws_type == val) {
                    return false;
                }
            }
            
            // Check path filter
            if let Some(path_values) = &path_filter {
                if let Some(info) = &ws.parsed_info {
                    if !path_values.iter().any(|&val| info.path.to_lowercase().contains(val)) {
                        return false;
                    }
                } else if !path_values.iter().any(|&val| ws.path.to_lowercase().contains(val)) {
                    return false;
                }
            }
            
            // Check tag filter
            if let Some(tag_values) = &tag_filter {
                if let Some(info) = &ws.parsed_info {
                    if !tag_values.iter().any(|&tag_val| 
                        info.tags.iter().any(|ws_tag| ws_tag.to_lowercase().contains(tag_val))) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            
            // Check existence filter
            if let Some(should_exist) = existing_filter {
                let exists = workspace_exists(ws);
                if exists != should_exist {
                    return false;
                }
            }
            
            true
        })
        .collect()
} 