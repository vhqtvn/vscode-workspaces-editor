use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urlencoding::decode;
use anyhow::{Result, anyhow};
use log::{debug, warn};

/// WorkspacePathInfo represents the fully parsed information from a workspace path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePathInfo {
    /// Original path as stored in the workspace
    pub original_path: String,
    /// The type of workspace (folder, file, workspace)
    pub workspace_type: WorkspaceType,
    /// For remote workspaces, the remote authority (e.g., SSH host)
    pub remote_authority: Option<String>,
    /// Host or computer name for remote workspaces
    pub remote_host: Option<String>,
    /// Username for remote connections
    pub remote_user: Option<String>,
    /// Port for remote connections
    pub remote_port: Option<u16>,
    /// Local path on the remote machine
    pub path: String,
    /// Container path for devcontainers
    pub container_path: Option<String>,
    /// Readable label
    pub label: Option<String>,
    /// Workspace tags (ssh, workspace, devcontainer, etc.)
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[derive(Default)]
pub enum WorkspaceType {
    #[default]
    Folder,
    File,
    Workspace,
}

/// Remote configuration data parsed from JSON
#[derive(Default)]
struct RemoteConfig {
    host: Option<String>,
    host_path: Option<String>,
    scheme: Option<String>,
    user: Option<String>,
    port: Option<u16>,
}

/// Parse a workspace path into a structured format with remote information
pub fn parse_workspace_path(path: &str) -> Result<WorkspacePathInfo> {
    debug!("Parsing workspace path: {}", path);
    
    let mut info: WorkspacePathInfo = WorkspacePathInfo {
        original_path: path.to_string(),
        workspace_type: WorkspaceType::Folder,
        remote_authority: None,
        remote_host: None,
        remote_user: None,
        remote_port: None,
        path: path.to_string(),
        container_path: None,
        label: None,
        tags: Vec::new(),
    };

    
    // Handle simple local folder path
    if !path.starts_with("vscode-remote://") {
        // check if it is a file or a folder
        if std::path::Path::new(path).is_file() {
            info.workspace_type = WorkspaceType::File;
            debug!("Parsed as local file: {}", path);
        } else {
            info.workspace_type = WorkspaceType::Folder;
            debug!("Parsed as local folder: {}", path);
        }
        return Ok(info);
    }
    
    // Parse remote workspace URI
    let uri_parts: Vec<&str> = path.splitn(2, "://").collect();
    if uri_parts.len() < 2 {
        return Err(anyhow!("Invalid URI format: {}", path));
    }
    
    // Split the remote part (ssh-remote+host) and the path
    let remote_parts: Vec<&str> = uri_parts[1].splitn(2, "/").collect();
    if remote_parts.len() < 2 {
        return Err(anyhow!("Invalid remote URI format: {}", path));
    }
    
    // Try to decode the remote authority part
    let remote_authority = match decode(remote_parts[0]) {
        Ok(decoded) => decoded.into_owned(),
        Err(_) => remote_parts[0].to_string(),
    };
        
    info.remote_authority = Some(remote_authority.clone());
    
    // Extract the path and ensure it starts with "/" for absolute paths
    let extracted_path = remote_parts[1].to_string();
    info.path = if extracted_path.starts_with('/') {
        extracted_path
    } else {
        format!("/{}", extracted_path)
    };
    
    info.tags.push("remote".to_string());
    
    info.workspace_type = WorkspaceType::Workspace;
    
    // Handle SSH remote
    if let Some(ssh_remote) = remote_authority.strip_prefix("ssh-remote+") {
        info.tags.push("ssh".to_string());
        
        // Try to decode hex-encoded JSON in SSH remote
        debug!("Decoding SSH remote authority: {}", ssh_remote);
        match decode_hex_if_needed(ssh_remote) {
            Ok(decoded_ssh_remote) => {
                // Handle JSON encoded SSH remote config
                if decoded_ssh_remote.starts_with("{") {
                    debug!("Parsing JSON SSH config: {}", decoded_ssh_remote);
                    match parse_json_remote_config(&decoded_ssh_remote) {
                        Ok(config) => {
                            let host_str = config.host.unwrap_or_else(|| decoded_ssh_remote.to_string());
                            info.remote_host = Some(host_str);
                            info.remote_user = config.user;
                            info.remote_port = config.port;
                            info.container_path = Some(info.path.clone());
                            if let Some(path_str) = config.host_path {
                                info.path = path_str;
                            }
                            
                            if let Some(scheme_str) = config.scheme {
                                info.tags.push(scheme_str);
                            }
                        },
                        Err(e) => {
                            warn!("Failed to parse SSH JSON config: {}", e);
                            // Try to parse from standard SSH format (user@host:port)
                            parse_ssh_remote_string(&decoded_ssh_remote, &mut info);
                        }
                    }
                } else {
                    // Regular SSH remote (user@host:port)
                    parse_ssh_remote_string(&decoded_ssh_remote, &mut info);
                }
            },
            Err(e) => {
                warn!("Failed to decode hex-encoded SSH remote: {}", e);
                parse_ssh_remote_string(ssh_remote, &mut info);
            }
        }
    }
    // Handle Dev Container remote
    else if let Some(container_remote) = remote_authority.strip_prefix("dev-container+") {
        info.tags.push("devcontainer".to_string());
        
        // Handle '@' separator in dev container remote
        let (config_hex, host) = if let Some(at_pos) = container_remote.rfind('@') {
            (&container_remote[..at_pos], Some(&container_remote[(at_pos + 1)..]))
        } else {
            (container_remote, None)
        };
        
        // Try to decode hex-encoded config
        match decode_hex_if_needed(config_hex) {
            Ok(decoded_config) => {
                if decoded_config.starts_with("{") {
                    debug!("Parsing JSON dev container config: {}", decoded_config);
                    match parse_json_remote_config(&decoded_config) {
                        Ok(config) => {
                            let host_str = match config.host {
                                Some(h) => h,
                                None => host.unwrap_or("").to_string(),
                            };
                            
                            if !host_str.is_empty() {
                                info.remote_host = Some(host_str);
                            }
                            
                            info.remote_user = config.user;
                            info.remote_port = config.port;
                            info.container_path = Some(info.path.clone());
                            
                            if let Some(path_str) = config.host_path {
                                info.path = path_str;
                            }
                            
                            if let Some(scheme_str) = config.scheme {
                                info.tags.push(scheme_str);
                            }
                        },
                        Err(e) => {
                            warn!("Failed to parse container JSON config: {}", e);
                            if let Some(h) = host {
                                info.remote_host = Some(h.to_string());
                                // Try to parse from standard SSH format (user@host:port)
                                if h.contains('@') {
                                    parse_ssh_remote_string(h, &mut info);
                                }
                            }
                        }
                    }
                } else if let Some(h) = host {
                    info.remote_host = Some(h.to_string());
                    // Try to parse from standard SSH format (user@host:port)
                    if h.contains('@') {
                        parse_ssh_remote_string(h, &mut info);
                    }
                }
            },
            Err(_) => {
                if let Some(h) = host {
                    info.remote_host = Some(h.to_string());
                    // Try to parse from standard SSH format (user@host:port)
                    if h.contains('@') {
                        parse_ssh_remote_string(h, &mut info);
                    }
                }
            }
        }
    }
    
    debug!("Parsed workspace info: {:?}", info);
    Ok(info)
}

/// Try to decode a hex-encoded string (especially for JSON config in remote URIs)
pub fn decode_hex_if_needed(input: &str) -> Result<String> {
    // Check if it might be hex encoded
    if input.chars().all(|c| c.is_ascii_hexdigit() || c == '{' || c == '}' || c == '"' || c == ':' || c == ',' || c == ' ') {
        // If it already starts with '{', assume it's JSON and not hex encoded
        if input.starts_with('{') {
            return Ok(input.to_string());
        }
        
        // Try to decode from hex
        let mut output = String::new();
        let mut chars = input.chars().peekable();
        
        while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
            if let (Some(d1), Some(d2)) = (c1.to_digit(16), c2.to_digit(16)) {
                let byte = ((d1 * 16) + d2) as u8;
                output.push(byte as char);
            } else {
                return Err(anyhow!("Invalid hex encoding"));
            }
        }
        
        if output.starts_with('{') {
            return Ok(output);
        }
    }
    
    // Return original string if not hex encoded or decoding failed
    Ok(input.to_string())
}

/// Parse JSON config found in remote paths
fn parse_json_remote_config(json_config: &str) -> Result<RemoteConfig> {
    let config: HashMap<String, serde_json::Value> = serde_json::from_str(json_config)?;
    
    let host = config.get("settings")
        .and_then(|settings| settings.get("host"))
        .and_then(|host| host.as_str())
        .map(String::from)
        .or_else(|| config.get("hostName")
            .and_then(|host| host.as_str())
            .map(String::from)
        );
    
    let host_path = config.get("hostPath")
        .and_then(|path| path.as_str())
        .map(String::from);
    
    let scheme = config.get("scheme")
        .and_then(|scheme| scheme.as_str())
        .map(String::from);
    
    let user = config.get("settings")
        .and_then(|settings| settings.get("user"))
        .and_then(|user| user.as_str())
        .map(String::from)
        .or_else(|| config.get("user")
            .and_then(|user| user.as_str())
            .map(String::from)
        );

    let port = config.get("settings")
        .and_then(|settings| settings.get("port"))
        .and_then(|port| port.as_u64())
        .map(|p| p as u16)
        .or_else(|| config.get("port")
            .and_then(|port| port.as_u64())
            .map(|p| p as u16));

    Ok(RemoteConfig {
        host,
        host_path,
        scheme,
        user,
        port,
    })
}

/// Parse SSH remote string and populate WorkspacePathInfo
fn parse_ssh_remote_string(remote_str: &str, info: &mut WorkspacePathInfo) {
    // Handle user@host or user@host:port or user@host:/path or user@host:port:/path format
    if let Some(at_pos) = remote_str.find('@') {
        let user = &remote_str[..at_pos];
        let host_part = &remote_str[(at_pos + 1)..];
        
        info.remote_user = Some(user.to_string());
        
        // Check if there's a colon after the host
        if let Some(colon_pos) = host_part.find(':') {
            let host = &host_part[..colon_pos];
            let after_colon = &host_part[(colon_pos + 1)..];
            
            info.remote_host = Some(host.to_string());
            
            // Try to determine if what follows the colon is a port, path, or port:path
            if let Some(second_colon_pos) = after_colon.find(':') {
                // Format: user@host:port:/path
                let port_str = &after_colon[..second_colon_pos];
                let path_part = &after_colon[(second_colon_pos + 1)..];
                
                if let Ok(port) = port_str.parse::<u16>() {
                    info.remote_port = Some(port);
                }
                
                if !path_part.is_empty() {
                    info.path = path_part.to_string();
                }
            } else if after_colon.parse::<u16>().is_ok() {
                // Format: user@host:port (no path)
                info.remote_port = Some(after_colon.parse::<u16>().unwrap());
            } else if after_colon.starts_with('/') || after_colon.starts_with('~') {
                // Format: user@host:/path (no port)
                info.path = after_colon.to_string();
            } else {
                // Could be either, try port first, then assume it's a relative path
                if let Ok(port) = after_colon.parse::<u16>() {
                    info.remote_port = Some(port);
                } else {
                    info.path = after_colon.to_string();
                }
            }
        } else {
            // Just host without port or path
            info.remote_host = Some(host_part.to_string());
        }
    } else {
        // No @ symbol, might be just host:path or host:port
        if let Some(colon_pos) = remote_str.find(':') {
            let host = &remote_str[..colon_pos];
            let after_colon = &remote_str[(colon_pos + 1)..];
            
            info.remote_host = Some(host.to_string());
            
            if let Ok(port) = after_colon.parse::<u16>() {
                info.remote_port = Some(port);
            } else {
                info.path = after_colon.to_string();
            }
        } else {
            // Just host without port or path
            info.remote_host = Some(remote_str.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_local_path() {
        let path = "/home/user/projects/myproject";
        let info = parse_workspace_path(path).unwrap();
        
        assert_eq!(info.original_path, path);
        assert_eq!(info.workspace_type, WorkspaceType::Folder);
        assert_eq!(info.path, path);
        assert!(info.remote_authority.is_none());
        assert!(info.remote_host.is_none());
        assert!(info.remote_user.is_none());
        assert!(info.remote_port.is_none());
        assert!(info.container_path.is_none());
        assert!(info.tags.is_empty());
    }
    
    #[test]
    fn test_parse_ssh_remote() {
        let path = "vscode-remote://ssh-remote+user@example.com/home/user/project";
        let info = parse_workspace_path(path).unwrap();
        
        assert_eq!(info.original_path, path);
        assert_eq!(info.workspace_type, WorkspaceType::Workspace);
        assert_eq!(info.path, "home/user/project");
        assert!(info.remote_authority.is_some());
        assert!(info.remote_host.is_some());
        assert_eq!(info.remote_user, Some("user".to_string()));
        assert!(info.remote_port.is_none());
        assert!(info.tags.contains(&"remote".to_string()));
        assert!(info.tags.contains(&"ssh".to_string()));

        // Test with port
        let path_with_port = "vscode-remote://ssh-remote+user@example.com:2222/home/user/project";
        let info_with_port = parse_workspace_path(path_with_port).unwrap();
        
        assert_eq!(info_with_port.remote_user, Some("user".to_string()));
        assert_eq!(info_with_port.remote_port, Some(2222));
    }
    
    #[test]
    fn test_parse_dev_container() {
        let path = "vscode-remote://dev-container+abc@hostname/container/path";
        let info = parse_workspace_path(path).unwrap();
        
        assert_eq!(info.original_path, path);
        assert_eq!(info.workspace_type, WorkspaceType::Workspace);
        assert_eq!(info.path, "container/path");
        assert!(info.remote_authority.is_some());
        assert!(info.remote_host.is_some());
        assert!(info.tags.contains(&"remote".to_string()));
        assert!(info.tags.contains(&"devcontainer".to_string()));
    }
    
    #[test]
    fn test_decode_hex() {
        // Test JSON input
        let json_input = "{\"host\":\"example.com\"}";
        let result = decode_hex_if_needed(json_input).unwrap();
        assert_eq!(result, json_input);
        
        // Test hex input representing {"host":"example.com"}
        let hex_input = "7b22686f7374223a226578616d706c652e636f6d227d";
        let result = decode_hex_if_needed(hex_input).unwrap();
        assert_eq!(result, "{\"host\":\"example.com\"}");
    }

    #[test]
    fn test_parse_ssh_remote_string() {
        // Test user@host format
        let mut info = WorkspacePathInfo {
            original_path: "test".to_string(),
            workspace_type: WorkspaceType::Workspace,
            remote_authority: None,
            remote_host: None,
            remote_user: None,
            remote_port: None,
            path: "original/path".to_string(),
            container_path: None,
            label: None,
            tags: Vec::new(),
        };
        
        parse_ssh_remote_string("user@host", &mut info);
        assert_eq!(info.remote_user, Some("user".to_string()));
        assert_eq!(info.remote_host, Some("host".to_string()));
        assert!(info.remote_port.is_none());
        assert_eq!(info.path, "original/path"); // Should remain unchanged
        
        // Test user@host:port format
        let mut info2 = info.clone();
        parse_ssh_remote_string("user@host:2222", &mut info2);
        assert_eq!(info2.remote_user, Some("user".to_string()));
        assert_eq!(info2.remote_host, Some("host".to_string()));
        assert_eq!(info2.remote_port, Some(2222));
        assert_eq!(info2.path, "original/path"); // Should remain unchanged
        
        // Test user@host:/path format
        let mut info3 = info.clone();
        parse_ssh_remote_string("user@host:/home/user/project", &mut info3);
        assert_eq!(info3.remote_user, Some("user".to_string()));
        assert_eq!(info3.remote_host, Some("host".to_string()));
        assert!(info3.remote_port.is_none());
        assert_eq!(info3.path, "/home/user/project"); // Should be updated
        
        // Test user@host:port:/path format
        let mut info4 = info.clone();
        parse_ssh_remote_string("user@host:2222:/home/user/project", &mut info4);
        assert_eq!(info4.remote_user, Some("user".to_string()));
        assert_eq!(info4.remote_host, Some("host".to_string()));
        assert_eq!(info4.remote_port, Some(2222));
        assert_eq!(info4.path, "/home/user/project"); // Should be updated
        
        // Test host:path format (no user)
        let mut info5 = WorkspacePathInfo {
            original_path: "test".to_string(),
            workspace_type: WorkspaceType::Workspace,
            remote_authority: None,
            remote_host: None,
            remote_user: None,
            remote_port: None,
            path: "original/path".to_string(),
            container_path: None,
            label: None,
            tags: Vec::new(),
        };
        parse_ssh_remote_string("host:/home/user/project", &mut info5);
        assert!(info5.remote_user.is_none());
        assert_eq!(info5.remote_host, Some("host".to_string()));
        assert!(info5.remote_port.is_none());
        assert_eq!(info5.path, "/home/user/project"); // Should be updated
    }
}
