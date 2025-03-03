use anyhow::{Context, Result};
use glob::glob;
use log::{debug, warn};
use std::fs;

use crate::workspaces::models::{Workspace, WorkspaceSource};
use crate::workspaces::paths::expand_tilde;

/// Get workspaces from workspace storage files
pub fn get_workspaces_from_storage(profile_path: &str) -> Result<Vec<Workspace>> {
    let profile_path = expand_tilde(profile_path)?;
    let storage_path = format!("{}/User/workspaceStorage/*/workspace.json", profile_path);
    
    let mut workspaces = Vec::new();
    
    for entry in glob(&storage_path)
        .context("Failed to read glob pattern")?
    {
        match entry {
            Ok(path) => {
                debug!("Reading workspace file: {:?}", path);
                
                // Get file metadata for fallback timestamp
                let metadata = match fs::metadata(&path) {
                    Ok(meta) => Some(meta),
                    Err(e) => {
                        warn!("Failed to read metadata for workspace file: {:?} - {}", path, e);
                        None
                    }
                };
                
                // Get the file modification time as a fallback for last_used
                let file_mtime = metadata
                    .and_then(|meta| meta.modified().ok())
                    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|duration| duration.as_secs() as i64 * 1000) // Convert to milliseconds
                    .unwrap_or(0);
                
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read workspace file: {:?}", path))?;
                
                // Get the ID from the parent directory name
                let id = path.parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                // Parse the workspace file
                let workspace_json: serde_json::Value = serde_json::from_str(&content)
                    .with_context(|| format!("Failed to parse workspace file: {:?}", path))?;
                
                if let Some(folder_uri) = workspace_json["folder"].as_str() {
                    // Remove the file:// prefix
                    let folder_path = folder_uri.replace("file://", "");
                    
                    // Get the storage path relative to the workspace storage directory
                    let relative_storage_path = path.to_string_lossy().to_string();
                    let storage_path_parts: Vec<&str> = relative_storage_path.split("workspaceStorage").collect();
                    let relative_path = if storage_path_parts.len() > 1 {
                        format!("workspaceStorage{}", storage_path_parts[1])
                    } else {
                        relative_storage_path
                    };
                    
                    let workspace = Workspace {
                        id,
                        name: None, // Will be filled from state.vscdb
                        path: folder_path,
                        last_used: file_mtime, // Use file modification time as fallback
                        storage_path: Some(relative_path.clone()),
                        sources: vec![WorkspaceSource::Storage(relative_path)],
                        parsed_info: None,
                    };
                    
                    workspaces.push(workspace);
                }
            },
            Err(e) => warn!("Failed to read workspace entry: {}", e),
        }
    }
    
    Ok(workspaces)
} 