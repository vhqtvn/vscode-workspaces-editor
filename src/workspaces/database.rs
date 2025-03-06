use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::workspaces::models::{Workspace, WorkspaceSource};
use crate::workspaces::paths::{generate_path_variations, normalize_path};

/// Get workspace names and last used times from state database
pub fn get_workspace_metadata(profile_path: &str, workspaces: &mut Vec<Workspace>) -> Result<()> {
    let main_db_path = format!("{}/User/state.vscdb", profile_path);
    info!("Checking for database at path: {}", main_db_path);
    
    // Extract the relative path to be used as source identifier
    let main_db_relative_path = if let Some(stripped) = main_db_path.strip_prefix(profile_path) {
        stripped.trim_start_matches('/').to_string()
    } else {
        "User/state.vscdb".to_string()
    };
    
    // Check if the main database file exists and get its size
    let main_db_exists = Path::new(&main_db_path).exists();
    let main_db_size = if main_db_exists {
        match fs::metadata(&main_db_path) {
            Ok(metadata) => metadata.len(),
            Err(_) => 0,
        }
    } else {
        0
    };
    
    info!("Main database file exists with size: {} bytes", main_db_size);
    
    // Also check the alternative database in the globalStorage directory
    let alt_db_path = format!("{}/User/globalStorage/state.vscdb", profile_path);
    
    // Extract the relative path for alternative database
    let alt_db_relative_path = if let Some(stripped) = alt_db_path.strip_prefix(profile_path) {
        stripped.trim_start_matches('/').to_string()
    } else {
        "User/global-state.vscdb".to_string()
    };
    
    info!("Checking alternative database path: {}", alt_db_path);
    
    let alt_db_exists = Path::new(&alt_db_path).exists();
    let alt_db_size = if alt_db_exists {
        match fs::metadata(&alt_db_path) {
            Ok(metadata) => metadata.len(),
            Err(_) => 0,
        }
    } else {
        0
    };
    
    info!("Alternative database file exists with size: {} bytes", alt_db_size);
    
    // Check and process both databases if they exist
    let mut main_processed = false;
    
    // Try to get workspace metadata from the main database if it exists and has content
    if main_db_exists && main_db_size > 0 {
        match get_workspace_metadata_from_db(&main_db_path, workspaces, &main_db_relative_path) {
            Ok(_) => {
                main_processed = true;
                info!("Successfully processed main database");
            },
            Err(e) => {
                warn!("Failed to process main database: {}", e);
            }
        }
    } else if main_db_exists {
        warn!("Main database file is empty");
    } else {
        warn!("Main database file does not exist");
    }
    
    // Now try the alternative database
    if alt_db_exists && alt_db_size > 0 {
        match get_workspace_metadata_from_db(&alt_db_path, workspaces, &alt_db_relative_path) {
            Ok(_) => {
                info!("Successfully processed alternative database");
                if main_processed {
                    info!("Data merged from both databases");
                } else {
                    info!("Using data only from alternative database");
                }
            },
            Err(e) => {
                warn!("Failed to process alternative database: {}", e);
                if !main_processed {
                    return Err(e);
                }
            }
        }
    } else if alt_db_exists {
        warn!("Alternative database file is empty");
    } else {
        warn!("Alternative database file does not exist");
        
        // If neither database was processed, return an error
        if !main_processed {
            return Err(anyhow!("No valid database files found"));
        }
    }

    Ok(())
}

/// Helper function to extract metadata from a database file
fn get_workspace_metadata_from_db(db_path: &str, workspaces: &mut Vec<Workspace>, db_source: &str) -> Result<()> {
    info!("Opening database connection: {}", db_path);
    let conn = match rusqlite::Connection::open(db_path) {
        Ok(conn) => {
            info!("Successfully opened database connection");
            conn
        },
        Err(e) => {
            warn!("Failed to open database: {}", e);
            return Ok(());
        }
    };
    
    // Get table names
    let mut table_names = Vec::new();
    let mut tables_stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
    let table_rows = tables_stmt.query_map([], |row| row.get::<_, String>(0))?;
    
    for name in table_rows.flatten() {
        table_names.push(name);
    }
    
    info!("Found tables in database: {:?}", table_names);
    
    if !table_names.contains(&"ItemTable".to_string()) {
        warn!("ItemTable not found in database, cannot retrieve workspace history");
        return Ok(());
    }
    
    info!("Looking for history.recentlyOpenedPathsList in ItemTable");
    
    // Try to find and process workspaces from the history.recentlyOpenedPathsList key
    match conn.query_row(
        "SELECT value FROM ItemTable WHERE key = ?",
        ["history.recentlyOpenedPathsList"],
        |row| row.get::<_, String>(0)
    ) {
        Ok(value) => {
            info!("Found history.recentlyOpenedPathsList entry");
            let count = process_workspace_rows(value, workspaces, db_source);
            info!("Processed {} workspaces from history.recentlyOpenedPathsList", count);
        }
        Err(e) => {
            warn!("Failed to retrieve history.recentlyOpenedPathsList from database: {}", e);
        }
    }
    
    Ok(())
}

// Helper function to process workspace rows from the database
// Returns the number of rows processed successfully
fn process_workspace_rows(rows: String, workspaces: &mut Vec<Workspace>, db_source: &str) -> usize {
    debug!("Processing history.recentlyOpenedPathsList");
    
    // Create a map of workspace paths to their indices
    let mut path_to_index = HashMap::new();
    for (i, workspace) in workspaces.iter().enumerate() {
        path_to_index.insert(workspace.path.clone(), i);
    }
    
    let mut processed_count = 0;
    
    match serde_json::from_str::<serde_json::Value>(&rows) {
        Ok(value) => {
            debug!("JSON structure: {}", value);
            
            // Check if the value contains an "entries" array
            if let Some(entries) = value.get("entries").and_then(|e| e.as_array()) {
                info!("Found entries array with {} entries", entries.len());
                
                for (i, entry) in entries.iter().enumerate() {
                    debug!("Processing entry {}: {:?}", i, entry);
                    
                    // Use db_source directly without adding "/entry-i" suffix
                    if process_workspace_entry(entry, workspaces, &mut path_to_index, db_source) {
                        processed_count += 1;
                    }
                }
            } else {
                warn!("Expected 'entries' array in history.recentlyOpenedPathsList but got: {}", value);
            }
        }
        Err(e) => {
            warn!("Failed to parse JSON from history.recentlyOpenedPathsList: {}", e);
        }
    }
    
    info!("Processed {} workspaces from history.recentlyOpenedPathsList", processed_count);
    processed_count
}

// Helper function to check if paths would match after normalization
#[allow(dead_code)]
fn check_path_matching(db_path: &str, workspace_paths: &[String]) -> bool {
    let normalized_db_path = normalize_path(db_path);
    
    debug!("Checking path matching for: {}", db_path);
    debug!("Normalized to: {}", normalized_db_path);
    
    // Try adding/removing file:// prefix
    let alt_path = if db_path.starts_with("file://") {
        db_path.replace("file://", "")
    } else {
        format!("file://{}", db_path)
    };
    
    debug!("Alternative path: {}", alt_path);
    
    // Show a sample of workspace paths for comparison
    let sample_paths = workspace_paths.iter().take(5).collect::<Vec<_>>();
    debug!("Sample workspace paths: {:?}", sample_paths);
    
    for workspace_path in workspace_paths {
        let normalized_workspace_path = normalize_path(workspace_path);
        
        if normalized_db_path == normalized_workspace_path {
            info!("Found exact match after normalization: {} == {}", 
                 normalized_db_path, normalized_workspace_path);
            return true;
        }
        
        // Check if the paths match ignoring case (for case-insensitive filesystems)
        if normalized_db_path.to_lowercase() == normalized_workspace_path.to_lowercase() {
            info!("Found case-insensitive match: {} ~= {}", 
                 normalized_db_path, normalized_workspace_path);
            return true;
        }
        
        // Check if one path is contained within the other
        if normalized_db_path.contains(&normalized_workspace_path) || 
           normalized_workspace_path.contains(&normalized_db_path) {
            info!("Found path containment: {} contains or is contained in {}", 
                 normalized_db_path, normalized_workspace_path);
            debug!("Path lengths - DB: {}, Workspace: {}", 
                  normalized_db_path.len(), normalized_workspace_path.len());
        }
    }
    
    false
}

/// Process a workspace entry from the database
fn process_workspace_entry(
    entry: &serde_json::Value,
    workspaces: &mut Vec<Workspace>,
    workspace_map: &mut HashMap<String, usize>,
    source_identifier: &str
) -> bool {
    let mut processed = false;
    
    // Extract the workspace path from potential fields: folderUri, fileUri, workspace
    let path = if let Some(folder_uri) = entry.get("folderUri").and_then(|u| u.as_str()) {
        debug!("Found folderUri: {}", folder_uri);
        Some(folder_uri)
    } else if let Some(file_uri) = entry.get("fileUri").and_then(|u| u.as_str()) {
        debug!("Found fileUri (skipping as it's a file, not a workspace): {}", file_uri);
        // Skip files, only process folders and workspaces
        return false;
    } else if let Some(workspace) = entry.get("workspace") {
        // This is a workspace entry with a workspace object
        if let Some(workspace_uri) = workspace.get("uri").and_then(|u| u.as_str()) {
            debug!("Found workspace uri: {}", workspace_uri);
            Some(workspace_uri)
        } else if let Some(config_path) = workspace.get("configPath").and_then(|p| p.as_str()) {
            debug!("Found workspace configPath: {}", config_path);
            Some(config_path)
        } else {
            warn!("Workspace entry missing uri and configPath: {:?}", workspace);
            None
        }
    } else {
        warn!("Entry is missing folderUri, fileUri, and workspace fields: {:?}", entry);
        None
    };
    
    if let Some(workspace_path) = path {
        // Extract name and last_used from the entry
        let name = entry.get("name").and_then(|n| n.as_str()).map(|s| s.to_string());
        let last_used = entry.get("lastUsed").and_then(|t| t.as_i64()).unwrap_or(0);

        // Process the workspace with the extracted data
        processed = process_workspace_details(workspace_path, name.unwrap_or_default().as_str(), last_used, workspaces, workspace_map, source_identifier);
    }
    
    processed
}

/// Process a workspace's details, creating or updating a workspace entry
fn process_workspace_details(
    workspace_path: &str, 
    workspace_name: &str, 
    workspace_last_used: i64, 
    workspaces: &mut Vec<Workspace>, 
    workspace_map: &mut HashMap<String, usize>,
    source_identifier: &str
) -> bool {
    debug!("Processing workspace path: {}", workspace_path);
    
    // Normalize the path
    let normalized_path = normalize_path(workspace_path);
    debug!("Normalized path: {}", normalized_path);
    
    // For remote paths, we need to match the full URI
    let normalized_workspace_path = if workspace_path.starts_with("vscode-remote://") {
        normalized_path.clone()
    } else {
        normalize_path(&normalized_path)
    };
    let path_variations = generate_path_variations(&normalized_workspace_path);
    
    // First try to find an exact match
    let mut found_idx = None;
    if let Some(&idx) = workspace_map.get(&normalized_workspace_path) {
        debug!("Found exact path match at index {}", idx);
        found_idx = Some(idx);
    } else {
        // Try with variations
        for variation in &path_variations {
            if let Some(&idx) = workspace_map.get(variation) {
                debug!("Found path variation match: {} at index {}", variation, idx);
                found_idx = Some(idx);
                break;
            }
        }
    }
    
    // Create a database source with the identifier
    let db_source = WorkspaceSource::Database(source_identifier.to_string());
    
    if let Some(idx) = found_idx {
        debug!("Updating workspace at index {}", idx);
        let workspace = &mut workspaces[idx];
        
        // Update name if provided and workspace doesn't already have one
        if !workspace_name.is_empty() && workspace.name.is_none() {
            workspace.name = Some(workspace_name.to_string());
        }
        
        // Only update last_used if the database has a newer timestamp
        if workspace_last_used > 0 && workspace.last_used < workspace_last_used {
            debug!("Setting last_used to: {}", workspace_last_used);
            workspace.last_used = workspace_last_used;
        }
        
        // Add the database source to the sources list if it's not already there
        if !workspace.sources.iter().any(|src| matches!(src, WorkspaceSource::Database(_))) {
            workspace.sources.push(db_source);
        }
        
        true
    } else {
        // If no matching workspace found in storage, create a new one from the database
        debug!("Creating new workspace from database: {}", normalized_workspace_path);
        
        // Generate a unique ID for the workspace
        let id = format!("db-{}", Uuid::new_v4());
        
        // Create a new workspace with default values
        let workspace = Workspace {
            id,
            name: if workspace_name.is_empty() { None } else { Some(workspace_name.to_string()) },
            path: normalized_workspace_path.clone(),
            last_used: workspace_last_used,
            storage_path: None,
            sources: vec![db_source],
            parsed_info: None,
        };
        
        // Add the new workspace to the list
        workspaces.push(workspace);
        
        // Update the map with the new index
        let new_idx = workspaces.len() - 1;
        workspace_map.insert(normalized_workspace_path, new_idx);
        
        true
    }
} 