use serde::{Deserialize, Serialize, Serializer};
use crate::workspaces::parser::WorkspacePathInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: Option<String>,
    pub path: String,
    pub last_used: i64,
    pub storage_path: Option<String>,
    #[serde(skip_deserializing)]
    #[serde(serialize_with = "serialize_sources")]
    pub sources: Vec<WorkspaceSource>,
    #[serde(skip_deserializing)]
    #[serde(serialize_with = "serialize_parsed_info")]
    pub parsed_info: Option<WorkspacePathInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkspaceSource {
    Storage(String),     // From workspace.json file with path
    Database(String),    // From state.vscdb with entry key
    Zed(String),         // From Zed's db.sqlite with channel name
}

impl Default for WorkspaceSource {
    fn default() -> Self {
        WorkspaceSource::Storage("unknown".to_string())
    }
}

pub fn serialize_sources<S>(sources: &[WorkspaceSource], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Format the sources in a readable way
    let formatted_sources: Vec<String> = sources.iter().map(|source| {
        match source {
            WorkspaceSource::Storage(path) => format!("Storage({})", path),
            WorkspaceSource::Database(key) => format!("Database({})", key),
            WorkspaceSource::Zed(channel) => format!("Zed({})", channel),
        }
    }).collect();
    
    // Serialize the formatted sources
    formatted_sources.serialize(serializer)
}

/// Serialize parsed workspace information in a more readable format
pub fn serialize_parsed_info<S>(parsed_info: &Option<WorkspacePathInfo>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match parsed_info {
        Some(info) => {
            // Create a struct with all the parsed information
            let parsed_data = serde_json::json!({
                "original_path": info.original_path,
                "type": format!("{:?}", info.workspace_type),
                "remote_authority": info.remote_authority,
                "remote_host": info.remote_host,
                "remote_user": info.remote_user,
                "remote_port": info.remote_port,
                "path": info.path,
                "container_path": info.container_path,
                "label": info.label,
                "tags": info.tags,
            });
            
            parsed_data.serialize(serializer)
        },
        None => {
            // If there's no parsed info, return null
            serde_json::Value::Null.serialize(serializer)
        }
    }
}

impl Workspace {
    /// Parse the workspace path and return detailed information
    pub fn parse_path(&mut self) -> Option<&WorkspacePathInfo> {
        if self.parsed_info.is_none() {
            match crate::workspaces::parser::parse_workspace_path(&self.path) {
                Ok(info) => {
                    self.parsed_info = Some(info);
                }
                Err(_) => {
                    return None;
                }
            }
        }
        self.parsed_info.as_ref()
    }
    
    /// Get the readable label for this workspace
    pub fn get_label(&mut self) -> String {
        if let Some(name) = &self.name {
            if !name.is_empty() {
                return name.clone();
            }
        }
        
        if let Some(info) = self.parse_path() {
            if let Some(label) = &info.label {
                if !label.is_empty() {
                    return label.clone();
                }
            }
            
            // For remote workspaces, show host and path
            if let Some(host) = &info.remote_host {
                let mut remote_part = String::new();
                
                // Add user if available
                if let Some(user) = &info.remote_user {
                    remote_part.push_str(user);
                    remote_part.push('@');
                }
                
                remote_part.push_str(host);
                
                // Add port if available
                if let Some(port) = info.remote_port {
                    remote_part.push_str(&format!(":{}", port));
                }
                
                return format!("{}: {}", remote_part, info.path);
            }
            
            return info.path.clone();
        }
        
        self.path.clone()
    }
    
    /// Get the workspace type (folder, file, workspace)
    pub fn get_type(&mut self) -> String {
        if let Some(info) = self.parse_path() {
            match info.workspace_type {
                crate::workspaces::parser::WorkspaceType::Folder => "folder",
                crate::workspaces::parser::WorkspaceType::File => "file",
                crate::workspaces::parser::WorkspaceType::Workspace => "workspace",
            }
        } else {
            "folder" // default to folder if parsing fails
        }.to_string()
    }
    
    /// Check if this is a remote workspace
    pub fn is_remote(&mut self) -> bool {
        if let Some(info) = self.parse_path() {
            info.remote_authority.is_some()
        } else {
            false
        }
    }
} 