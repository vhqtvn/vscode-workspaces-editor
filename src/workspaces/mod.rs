// Re-export all public items from submodules
mod error;
mod models;
mod storage;
mod database;
mod paths;
mod utils;
pub mod parser;

// Public exports
pub use models::Workspace;
pub use models::WorkspaceSource;
pub use paths::{get_default_profile_path, get_known_vscode_paths};
pub use utils::{workspace_exists, extract_folder_basename};

// Public API
pub use api::{
    get_workspaces,
    delete_workspace,
};

mod api {
    use anyhow::{Context, Result};
    use log::{info, warn, debug};
    
    use crate::workspaces::models::{Workspace, WorkspaceSource};
    use crate::workspaces::paths::{self, expand_tilde};
    use crate::workspaces::storage::get_workspaces_from_storage;
    use crate::workspaces::database::get_workspace_metadata;
    use crate::workspaces::utils::{process_workspaces, filter_workspaces};

    /// Get all workspaces from the VSCode profile
    pub fn get_workspaces(profile_path: &str) -> Result<Vec<Workspace>> {
        info!("Getting workspaces from: {}", profile_path);
        
        // Get workspaces from storage
        let mut workspaces = get_workspaces_from_storage(profile_path)?;
        
        // Try to update metadata from database and add any new workspaces
        let profile_path = expand_tilde(profile_path)?;
        
        // Update metadata from database if available and add any new workspaces found only in database
        if let Err(e) = get_workspace_metadata(&profile_path, &mut workspaces) {
            warn!("Failed to get workspace metadata from database: {}", e);
        }
        
        // Parse workspace paths to extract additional information
        if let Err(e) = process_workspaces(&mut workspaces) {
            warn!("Failed to process workspace paths: {}", e);
        }
        
        // Sort by last used time (descending)
        workspaces.sort_by(|a, b| b.last_used.cmp(&a.last_used));
        
        info!("Found {} workspaces in profile", workspaces.len());
        Ok(workspaces)
    }

    /// Search workspaces using filtering criteria
    #[allow(dead_code)]
    pub fn search_workspaces(profile_path: &str, query: &str) -> Result<Vec<Workspace>> {
        info!("Searching workspaces in profile '{}' with query: '{}'", profile_path, query);
        
        // First get all workspaces
        let mut all_workspaces = get_workspaces(profile_path)?;
        
        // Apply the filter
        let filtered_workspaces = filter_workspaces(&mut all_workspaces, query);
        
        // Convert the filtered references to owned instances
        let filtered_results: Vec<Workspace> = filtered_workspaces
            .into_iter()
            .cloned()
            .collect();
        
        info!("Found {} matching workspaces", filtered_results.len());
        Ok(filtered_results)
    }
    
    /// Delete a workspace from VSCode
    pub fn delete_workspace(profile_path: &str, workspaces: &[Workspace]) -> Result<bool> {
        if workspaces.is_empty() {
            info!("No workspaces to delete");
            return Ok(true);
        }
        
        info!("Attempting to delete {} workspaces from profile {}", workspaces.len(), profile_path);
        let profile_path = expand_tilde(profile_path)?;
        
        let mut success = true;
        let mut deleted_count = 0;
        
        // Process each workspace
        for workspace in workspaces {
            info!("Processing workspace: {} ({})", workspace.id, workspace.path);
            
            // Handle each source for the workspace
            for source in &workspace.sources {
                match source {
                    WorkspaceSource::Storage(storage_path) => {
                        // For storage, we need to delete the folder in workspaceStorage
                        if let Some(storage_dir) = build_storage_dir_path(&profile_path, storage_path) {
                            if let Err(e) = delete_storage_workspace(&storage_dir) {
                                warn!("Failed to delete storage workspace at {}: {}", storage_dir, e);
                                success = false;
                            } else {
                                info!("Successfully deleted storage workspace at {}", storage_dir);
                                deleted_count += 1;
                            }
                        } else {
                            warn!("Could not determine storage directory for {}", storage_path);
                            success = false;
                        }
                    },
                    WorkspaceSource::Database(db_source) => {
                        // For database, we need to update the JSON in the database
                        // Parse the source to determine which database to use
                        if let Some((db_path, _)) = parse_db_source(&profile_path, db_source) {
                            if let Err(e) = delete_database_workspace(&db_path, &workspace.path) {
                                warn!("Failed to delete workspace {} from database {}: {}", 
                                      workspace.path, db_path, e);
                                success = false;
                            } else {
                                info!("Successfully removed workspace {} from database {}", 
                                      workspace.path, db_path);
                                deleted_count += 1;
                            }
                        } else {
                            warn!("Could not determine database path from source: {}", db_source);
                            success = false;
                        }
                    }
                }
            }
        }
        
        info!("Deleted {} workspace sources", deleted_count);
        Ok(success)
    }
    
    // Helper function to build the full path to a workspace storage directory
    fn build_storage_dir_path(profile_path: &str, storage_path: &str) -> Option<String> {
        // Extract the workspace ID from the storage path
        // Expected format: workspaceStorage/WORKSPACE_ID/workspace.json
        let parts: Vec<&str> = storage_path.split('/').collect();
        if parts.len() >= 2 && parts[0] == "workspaceStorage" {
            let workspace_id = parts[1];
            return Some(format!("{}/User/workspaceStorage/{}", profile_path, workspace_id));
        }
        None
    }
    
    // Helper function to delete a workspace storage directory
    fn delete_storage_workspace(storage_dir: &str) -> Result<()> {
        info!("Deleting storage directory: {}", storage_dir);
        
        if !std::path::Path::new(storage_dir).exists() {
            warn!("Storage directory does not exist: {}", storage_dir);
            return Ok(());
        }
        
        // Remove the directory and all its contents
        std::fs::remove_dir_all(storage_dir)
            .with_context(|| format!("Failed to delete storage directory: {}", storage_dir))?;
        
        Ok(())
    }
    
    // Helper function to parse a database source string
    fn parse_db_source(profile_path: &str, db_source: &str) -> Option<(String, String)> {
        // Expected format: User/state.vscdb or User/globalStorage/state.vscdb
        // Build the full database path
        let full_db_path = format!("{}/{}", profile_path, db_source);
        Some((full_db_path, String::new()))
    }
    
    // Helper function to delete a workspace from a database
    fn delete_database_workspace(db_path: &str, workspace_path: &str) -> Result<()> {
        info!("Deleting workspace {} from database: {}", workspace_path, db_path);
        
        // Check if the database exists
        if !std::path::Path::new(db_path).exists() {
            warn!("Database file does not exist: {}", db_path);
            return Ok(());
        }
        
        // Open the database connection
        let conn = rusqlite::Connection::open(db_path)
            .with_context(|| format!("Failed to open database: {}", db_path))?;
        
        // Check if the ItemTable exists
        let table_exists: bool = conn.query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='ItemTable'",
            [],
            |_| Ok(true)
        ).unwrap_or(false);
        
        if !table_exists {
            warn!("ItemTable not found in database: {}", db_path);
            return Ok(());
        }
        
        // Get the history.recentlyOpenedPathsList entry
        let json_value: String = match conn.query_row(
            "SELECT value FROM ItemTable WHERE key = ?",
            ["history.recentlyOpenedPathsList"],
            |row| row.get(0)
        ) {
            Ok(value) => value,
            Err(e) => {
                warn!("Failed to retrieve history.recentlyOpenedPathsList: {}", e);
                return Ok(());
            }
        };
        
        // Parse the JSON
        let mut json: serde_json::Value = match serde_json::from_str(&json_value) {
            Ok(parsed) => parsed,
            Err(e) => {
                warn!("Failed to parse JSON from database: {}", e);
                return Ok(());
            }
        };
        
        // Check if there's an entries array
        let entries_modified = if let Some(entries) = json.get_mut("entries").and_then(|e| e.as_array_mut()) {
            // The normalized path we're looking to filter out
            let normalized_path = paths::normalize_path(workspace_path);
            debug!("Looking to remove paths matching: {}", normalized_path);
            
            // Count original entries for comparison
            let original_count = entries.len();
            
            // We'll collect indices to remove
            let mut indices_to_remove = Vec::new();
            
            // Find entries with matching paths
            for (i, entry) in entries.iter().enumerate() {
                let entry_path = if let Some(folder_uri) = entry.get("folderUri").and_then(|u| u.as_str()) {
                    Some(folder_uri)
                } else if let Some(workspace) = entry.get("workspace") {
                    if let Some(uri) = workspace.get("uri").and_then(|u| u.as_str()) {
                        Some(uri)
                    } else {
                        workspace.get("configPath").and_then(|p| p.as_str())
                    }
                } else {
                    None
                };
                
                if let Some(path) = entry_path {
                    let normalized_entry_path = paths::normalize_path(path);
                    if normalized_entry_path == normalized_path {
                        debug!("Found matching entry at index {}: {}", i, path);
                        indices_to_remove.push(i);
                    }
                }
            }
            
            // Remove indices in reverse order to maintain correct positions
            indices_to_remove.sort_unstable_by(|a, b| b.cmp(a));
            for idx in indices_to_remove {
                entries.remove(idx);
            }
            
            // Return whether we modified anything
            original_count > entries.len()
        } else {
            warn!("No entries array found in history.recentlyOpenedPathsList");
            false
        };
        
        // Only update the database if we actually removed something
        if entries_modified {
            // Serialize the updated JSON back to a string
            let updated_json = match serde_json::to_string(&json) {
                Ok(serialized) => serialized,
                Err(e) => {
                    warn!("Failed to serialize updated JSON: {}", e);
                    return Ok(());
                }
            };
            
            // Update the database entry
            match conn.execute(
                "UPDATE ItemTable SET value = ? WHERE key = ?",
                [&updated_json, "history.recentlyOpenedPathsList"]
            ) {
                Ok(rows) => {
                    if rows > 0 {
                        info!("Successfully updated database");
                    } else {
                        warn!("No rows were updated in the database");
                    }
                },
                Err(e) => {
                    warn!("Failed to update database: {}", e);
                    return Err(anyhow::anyhow!("Failed to update database: {}", e));
                }
            }
        } else {
            info!("No matching entries found in database to remove");
        }
        
        Ok(())
    }
} 