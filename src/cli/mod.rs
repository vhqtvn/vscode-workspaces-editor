use crate::workspaces::Workspace;
use crate::workspaces::WorkspaceSource;
use anyhow::Result;
use std::io::{self, Write};
use std::process::Command;

/// List workspaces in the specified format
pub fn list_workspaces(workspaces: &[Workspace], format: &str) -> Result<()> {
    match format.to_lowercase().as_str() {
        "json" => output_json(workspaces)?,
        _ => output_text(workspaces)?,
    }
    
    Ok(())
}

/// Output workspaces as formatted text
fn output_text(workspaces: &[Workspace]) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    
    if workspaces.is_empty() {
        writeln!(handle, "No workspaces found.")?;
        return Ok(());
    }
    
    writeln!(handle, "Found {} workspaces:", workspaces.len())?;
    writeln!(handle, "{:-<80}", "")?;
    
    for (i, workspace) in workspaces.iter().enumerate() {
        writeln!(handle, "{:3}. ID: {}", i + 1, workspace.id)?;
        writeln!(handle, "     Name: {}", workspace.name.as_deref().unwrap_or("N/A"))?;
        
        // Display parsed path if available, otherwise original path
        let display_path = if let Some(parsed_info) = &workspace.parsed_info {
            parsed_info.path.clone()
        } else {
            workspace.path.clone()
        };
        writeln!(handle, "     Path: {}", display_path)?;
        
        // Display parsed data
        if let Some(parsed_info) = &workspace.parsed_info {
            writeln!(handle, "     Original Path: {}", parsed_info.original_path)?;
            writeln!(handle, "     Type: {:?}", parsed_info.workspace_type)?;
            
            if let Some(label) = &parsed_info.label {
                writeln!(handle, "     Label: {}", label)?;
            }
            
            if let Some(remote_auth) = &parsed_info.remote_authority {
                writeln!(handle, "     Remote Authority: {}", remote_auth)?;
            }
            
            if let Some(remote_host) = &parsed_info.remote_host {
                writeln!(handle, "     Remote Host: {}", remote_host)?;
            }
            
            if let Some(remote_user) = &parsed_info.remote_user {
                writeln!(handle, "     Remote User: {}", remote_user)?;
            }
            
            if let Some(remote_port) = &parsed_info.remote_port {
                writeln!(handle, "     Remote Port: {}", remote_port)?;
            }

            if !parsed_info.tags.is_empty() {
                writeln!(handle, "     Tags: {}", parsed_info.tags.join(", "))?;
            }
        }
        
        if workspace.last_used > 0 {
            let last_used = chrono::DateTime::from_timestamp(workspace.last_used / 1000, 0)
                .map(|dt| {
                    let now = chrono::Utc::now();
                    let duration = now.signed_duration_since(dt);
                    
                    if duration.num_days() > 365 {
                        dt.format("%Y-%m-%d %H:%M:%S").to_string()
                    } else if duration.num_days() > 30 {
                        format!("{} months ago", duration.num_days() / 30)
                    } else if duration.num_days() > 0 {
                        format!("{} days ago", duration.num_days())
                    } else if duration.num_hours() > 0 {
                        format!("{} hours ago", duration.num_hours())
                    } else if duration.num_minutes() > 0 {
                        format!("{} minutes ago", duration.num_minutes())
                    } else {
                        "just now".to_string()
                    }
                })
                .unwrap_or_else(|| "Unknown".to_string());
            
            writeln!(handle, "     Last Used: {}", last_used)?;
        } else {
            writeln!(handle, "     Last Used: Unknown")?;
        }
        
        // Display each source with its details
        writeln!(handle, "     Sources:")?;
        if workspace.sources.is_empty() {
            writeln!(handle, "       None")?;
        } else {
            for source in &workspace.sources {
                match source {
                    WorkspaceSource::Storage(path) => 
                        writeln!(handle, "       Storage: {}", path)?,
                    WorkspaceSource::Database(key) => 
                        writeln!(handle, "       Database: {}", key)?,
                }
            }
        }
        
        writeln!(handle, "{:-<80}", "")?;
    }
    
    Ok(())
}

/// Output workspaces as JSON
fn output_json(workspaces: &[Workspace]) -> Result<()> {
    // Create a more detailed representation with original path explicitly included
    let workspace_details: Vec<serde_json::Value> = workspaces.iter().map(|workspace| {
        // Determine the path to display - use parsed path if available, otherwise original path
        let display_path = if let Some(parsed_info) = &workspace.parsed_info {
            parsed_info.path.clone()
        } else {
            workspace.path.clone()
        };
        
        let mut json_workspace = serde_json::json!({
            "id": workspace.id,
            "name": workspace.name,
            "path": display_path,
            "last_used": workspace.last_used,
            "last_used_human": if workspace.last_used > 0 {
                chrono::DateTime::from_timestamp(workspace.last_used / 1000, 0)
                    .map(|dt| {
                        let now = chrono::Utc::now();
                        let duration = now.signed_duration_since(dt);
                        
                        if duration.num_days() > 365 {
                            dt.format("%Y-%m-%d %H:%M:%S").to_string()
                        } else if duration.num_days() > 30 {
                            format!("{} months ago", duration.num_days() / 30)
                        } else if duration.num_days() > 0 {
                            format!("{} days ago", duration.num_days())
                        } else if duration.num_hours() > 0 {
                            format!("{} hours ago", duration.num_hours())
                        } else if duration.num_minutes() > 0 {
                            format!("{} minutes ago", duration.num_minutes())
                        } else {
                            "just now".to_string()
                        }
                    })
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Unknown".to_string()
            },
            "sources": workspace.sources,
        });
        
        // Add parsed_info with original_path explicitly
        if let Some(parsed_info) = &workspace.parsed_info {
            json_workspace["original_path"] = serde_json::Value::String(parsed_info.original_path.clone());
            json_workspace["workspace_type"] = serde_json::Value::String(format!("{:?}", parsed_info.workspace_type));
            
            if let Some(remote_authority) = &parsed_info.remote_authority {
                json_workspace["remote_authority"] = serde_json::Value::String(remote_authority.clone());
            }
            
            if let Some(remote_host) = &parsed_info.remote_host {
                json_workspace["remote_host"] = serde_json::Value::String(remote_host.clone());
            }
            
            if let Some(remote_user) = &parsed_info.remote_user {
                json_workspace["remote_user"] = serde_json::Value::String(remote_user.clone());
            }
            
            if let Some(remote_port) = &parsed_info.remote_port {
                json_workspace["remote_port"] = serde_json::Value::Number((*remote_port).into());
            }
            
            if let Some(container_path) = &parsed_info.container_path {
                json_workspace["container_path"] = serde_json::Value::String(container_path.clone());
            }
            
            if let Some(label) = &parsed_info.label {
                json_workspace["label"] = serde_json::Value::String(label.clone());
            }
            
            if !parsed_info.tags.is_empty() {
                json_workspace["tags"] = serde_json::Value::Array(
                    parsed_info.tags.iter()
                        .map(|tag| serde_json::Value::String(tag.clone()))
                        .collect()
                );
            }
        }
        
        json_workspace
    }).collect();
    
    let json = serde_json::to_string_pretty(&workspace_details)?;
    println!("{}", json);
    Ok(())
}

/// Open a workspace with VSCode
pub fn open_workspace(path: &str) -> Result<()> {
    // Determine the appropriate command to use based on the platform
    #[cfg(target_os = "windows")]
    let code_command = "code";
    
    #[cfg(target_os = "macos")]
    let code_command = "code";
    
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let code_command = "code";
    
    // Open the workspace with VSCode
    match Command::new(code_command)
        .arg(path)
        .spawn() {
            Ok(_) => {
                println!("Opening workspace in VSCode: {}", path);
                Ok(())
            },
            Err(e) => Err(anyhow::anyhow!("Failed to open workspace: {}", e)),
        }
} 