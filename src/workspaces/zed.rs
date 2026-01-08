use anyhow::{Context, Result};
use home::home_dir;
use log::{debug, info, warn};
use rusqlite::Connection;
use std::path::PathBuf;

use crate::workspaces::{
    models::{Workspace, WorkspaceSource},
    parser::WorkspacePathInfo,
};

/// Profile name for the Zed workspace source
pub const ZED_PROFILE_NAME: &str = "::zed";

/// Zed channel directories to check
const ZED_CHANNELS: &[&str] = &["0-stable", "0-preview", "0-nightly", "0-dev"];

/// Get the default Zed database path for the current platform
fn get_zed_db_path() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        return Ok(home.join("Library/Application Support/Zed/db"));
    }

    #[cfg(target_os = "windows")]
    {
        let local_app_data =
            std::env::var("LOCALAPPDATA").context("LOCALAPPDATA environment variable not found")?;
        return Ok(PathBuf::from(local_app_data).join("Zed/db"));
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // Try XDG_DATA_HOME first, fall back to ~/.local/share
        if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
            return Ok(PathBuf::from(xdg_data_home).join("zed/db"));
        }

        let home = home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        return Ok(home.join(".local/share/zed/db"));
    }
}

/// Get all Zed workspaces from all available channels
pub fn get_zed_workspaces() -> Result<Vec<Workspace>> {
    let mut all_workspaces = Vec::new();

    let zed_db_path = get_zed_db_path()?;
    info!("Looking for Zed databases in: {}", zed_db_path.display());

    if !zed_db_path.exists() {
        debug!(
            "Zed database directory does not exist: {}",
            zed_db_path.display()
        );
        return Ok(all_workspaces);
    }

    // Check each channel directory
    for channel in ZED_CHANNELS {
        let channel_path = zed_db_path.join(channel);

        if !channel_path.exists() {
            debug!(
                "Zed channel directory does not exist: {}",
                channel_path.display()
            );
            continue;
        }

        let db_file = channel_path.join("db.sqlite");

        if !db_file.exists() {
            debug!("Zed database file does not exist: {}", db_file.display());
            continue;
        }

        info!(
            "Found Zed database for channel '{}': {}",
            channel,
            db_file.display()
        );

        match get_workspaces_from_db(&db_file, channel) {
            Ok(mut workspaces) => {
                info!(
                    "Found {} workspaces in Zed channel '{}'",
                    workspaces.len(),
                    channel
                );
                all_workspaces.append(&mut workspaces);
            }
            Err(e) => {
                warn!(
                    "Failed to read workspaces from Zed database {}: {}",
                    db_file.display(),
                    e
                );
            }
        }
    }

    Ok(all_workspaces)
}

/// Get workspaces from a specific Zed database file
fn get_workspaces_from_db(db_path: &PathBuf, channel: &str) -> Result<Vec<Workspace>> {
    let mut workspaces = Vec::new();

    let conn = Connection::open(db_path)
        .with_context(|| format!("Failed to open Zed database: {}", db_path.display()))?;

    // Check if the workspaces table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='workspaces'",
            [],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if !table_exists {
        debug!("workspaces table not found in Zed database");
        return Ok(workspaces);
    }

    // Query workspaces with optional remote connection details
    let mut stmt = conn.prepare(
        "SELECT w.workspace_id, w.paths, w.remote_connection_id, w.timestamp,
                r.id, r.kind, r.host, r.port, r.user
         FROM workspaces w
         LEFT JOIN remote_connections r ON w.remote_connection_id = r.id",
    )?;

    let workspace_rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,            // workspace_id
            row.get::<_, Option<String>>(1)?, // paths (JSON array)
            row.get::<_, Option<i64>>(2)?,    // remote_connection_id
            row.get::<_, String>(3)?,         // timestamp
            row.get::<_, Option<i64>>(4)?,    // remote_connections.id
            row.get::<_, Option<String>>(5)?, // remote_connections.kind
            row.get::<_, Option<String>>(6)?, // remote_connections.host
            row.get::<_, Option<u16>>(7)?,    // remote_connections.port
            row.get::<_, Option<String>>(8)?, // remote_connections.user
        ))
    })?;

    for row in workspace_rows {
        let (
            workspace_id,
            paths_opt,
            _remote_connection_id,
            timestamp_str,
            _remote_id,
            remote_kind,
            remote_host,
            remote_port,
            remote_user,
        ) = row?;

        // Parse timestamp - Zed stores timestamps in "YYYY-MM-DD HH:MM:SS" format
        let timestamp =
            match chrono::NaiveDateTime::parse_from_str(&timestamp_str, "%Y-%m-%d %H:%M:%S") {
                Ok(dt) => dt.and_utc().timestamp_millis(),
                Err(e) => {
                    warn!("Failed to parse timestamp '{}': {}", timestamp_str, e);
                    0 // Default to 0 if parsing fails
                }
            };

        // The paths column contains a simple path string, not a JSON array
        let primary_path = match paths_opt {
            Some(path) => path,
            None => {
                // If paths is NULL, it might be a remote workspace without local paths
                // We'll handle this by checking if it's a remote workspace
                let is_remote = remote_kind.is_some() || remote_host.is_some();
                if is_remote {
                    "/".to_string()
                } else {
                    debug!("Skipping Zed workspace {} with no paths", workspace_id);
                    continue;
                }
            }
        };

        // Determine if this is a remote workspace
        let is_remote = remote_kind.is_some() || remote_host.is_some();

        if primary_path.is_empty() && !is_remote {
            debug!("Skipping Zed workspace {} with empty path", workspace_id);
            continue;
        }

        let mut parsed_info = None;

        // Build the workspace path
        let workspace_path = if is_remote {
            // For remote workspaces, construct a vscode-remote style URI
            if let (Some(host), Some(kind)) = (&remote_host, &remote_kind) {
                let mut uri = format!("vscode-remote://{}+", kind);

                if let Some(user) = &remote_user {
                    uri.push_str(user);
                    uri.push('@');
                }

                uri.push_str(host);
                let mut remote_authority = host.clone();

                if let Some(port) = remote_port {
                    uri.push(':');
                    uri.push_str(&port.to_string());
                    remote_authority = format!("{}:{}", host, port);
                }

                uri.push_str(&primary_path);
                parsed_info = Some(WorkspacePathInfo {
                    original_path: primary_path.clone(),
                    workspace_type: crate::workspaces::parser::WorkspaceType::Workspace,
                    remote_authority: Some(remote_authority),
                    remote_host,
                    remote_user,
                    remote_port,
                    path: primary_path.clone(),
                    container_path: None,
                    label: None,
                    tags: vec!["remote".to_string(), kind.to_string()],
                });
                uri
            } else {
                primary_path
            }
        } else {
            parsed_info = Some(WorkspacePathInfo {
                original_path: primary_path.clone(),
                workspace_type: crate::workspaces::parser::WorkspaceType::Workspace,
                remote_authority: None,
                remote_host: None,
                remote_user: None,
                remote_port: None,
                path: primary_path.clone(),
                container_path: None,
                label: None,
                tags: vec![],
            });
            primary_path
        };

        // Create the workspace
        let workspace = Workspace {
            id: workspace_id.to_string(),
            name: None,
            path: workspace_path,
            last_used: timestamp,
            storage_path: None,
            sources: vec![WorkspaceSource::Zed(channel.to_string())],
            parsed_info,
        };

        workspaces.push(workspace);
    }

    Ok(workspaces)
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, NaiveDateTime, Timelike};

    /// Test parsing of Zed timestamp format "YYYY-MM-DD HH:MM:SS"
    #[test]
    fn test_parse_zed_timestamp() {
        // Test the expected format from Zed
        let timestamp_str = "2025-06-27 16:20:06";

        let result = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S");

        assert!(result.is_ok(), "Failed to parse timestamp: {:?}", result);

        let dt = result.unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 27);
        assert_eq!(dt.hour(), 16);
        assert_eq!(dt.minute(), 20);
        assert_eq!(dt.second(), 6);

        // Verify it converts to milliseconds correctly
        let timestamp_millis = dt.and_utc().timestamp_millis();
        assert!(timestamp_millis > 0, "Timestamp should be positive");
    }

    /// Test parsing various valid timestamps
    #[test]
    fn test_parse_various_timestamps() {
        let test_cases = vec![
            ("2025-01-01 00:00:00", 2025, 1, 1, 0, 0, 0),
            ("2025-12-31 23:59:59", 2025, 12, 31, 23, 59, 59),
            ("2024-02-29 12:30:45", 2024, 2, 29, 12, 30, 45), // Leap year
            ("2023-06-15 08:30:00", 2023, 6, 15, 8, 30, 0),
        ];

        for (timestamp_str, year, month, day, hour, minute, second) in test_cases {
            let result = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S");
            assert!(
                result.is_ok(),
                "Failed to parse timestamp '{}': {:?}",
                timestamp_str,
                result
            );

            let dt = result.unwrap();
            assert_eq!(dt.year(), year, "Year mismatch for '{}'", timestamp_str);
            assert_eq!(dt.month(), month, "Month mismatch for '{}'", timestamp_str);
            assert_eq!(dt.day(), day, "Day mismatch for '{}'", timestamp_str);
            assert_eq!(dt.hour(), hour, "Hour mismatch for '{}'", timestamp_str);
            assert_eq!(
                dt.minute(),
                minute,
                "Minute mismatch for '{}'",
                timestamp_str
            );
            assert_eq!(
                dt.second(),
                second,
                "Second mismatch for '{}'",
                timestamp_str
            );
        }
    }

    /// Test that invalid timestamps fail gracefully
    #[test]
    fn test_parse_invalid_timestamps() {
        let invalid_cases = vec![
            "",                    // Empty string
            "2025-06-27",          // Missing time
            "16:20:06",            // Missing date
            "2025/06/27 16:20:06", // Wrong date separator
            "2025-06-27T16:20:06", // RFC 3339 format (should fail)
            "not-a-timestamp",     // Garbage
            "2025-13-01 00:00:00", // Invalid month
            "2025-02-30 00:00:00", // Invalid day
            "2025-06-27 25:00:00", // Invalid hour
            "2025-06-27 16:60:00", // Invalid minute
            "2025-06-27 16:20:61", // Invalid second (61 is out of range)
        ];

        for timestamp_str in invalid_cases {
            let result = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S");
            assert!(
                result.is_err(),
                "Expected failure for invalid timestamp '{}', but got: {:?}",
                timestamp_str,
                result
            );
        }
    }

    /// Test timestamp conversion to milliseconds
    #[test]
    fn test_timestamp_to_milliseconds() {
        let timestamp_str = "2025-06-27 16:20:06";
        let dt = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S").unwrap();
        let millis = dt.and_utc().timestamp_millis();

        // Verify the timestamp is reasonable (2025-06-27 should be around 1751000000000 ms)
        assert!(
            millis > 1_750_000_000_000,
            "Timestamp too small: {}",
            millis
        );
        assert!(
            millis < 2_000_000_000_000,
            "Timestamp too large: {}",
            millis
        );
    }

    /// Test edge cases
    #[test]
    fn test_edge_cases() {
        // Unix epoch (1970-01-01)
        let epoch = "1970-01-01 00:00:00";
        let result = NaiveDateTime::parse_from_str(epoch, "%Y-%m-%d %H:%M:%S");
        assert!(result.is_ok());
        let millis = result.unwrap().and_utc().timestamp_millis();
        assert_eq!(millis, 0);

        // Far future date
        let future = "2099-12-31 23:59:59";
        let result = NaiveDateTime::parse_from_str(future, "%Y-%m-%d %H:%M:%S");
        assert!(result.is_ok());
        let millis = result.unwrap().and_utc().timestamp_millis();
        assert!(millis > 4_000_000_000_000);
    }
}
