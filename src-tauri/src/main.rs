// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use vscode_workspaces_editor::workspaces;
use vscode_workspaces_editor::workspaces::Workspace;
use vscode_workspaces_editor::workspaces::get_known_vscode_paths as get_known_vscode_paths_impl;

#[tauri::command]
async fn get_workspaces(profile_path: String) -> Result<Vec<Workspace>, String> {
    workspaces::get_workspaces(&profile_path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_workspace(_profile_path: String, _workspace_path: String) -> Result<bool, String> {
    Ok(true) // TODO: Implement add_workspace functionality
}

#[tauri::command]
async fn edit_workspace(_profile_path: String, _workspace_id: String, _new_name: String) -> Result<bool, String> {
    Ok(true) // TODO: Implement edit_workspace functionality
}

#[tauri::command]
async fn delete_workspace(profile_path: String, workspace_id: String) -> Result<bool, String> {
    // Find the workspace with the given ID
    let workspaces = workspaces::get_workspaces(&profile_path).map_err(|e| e.to_string())?;
    
    let workspace = workspaces.iter()
        .find(|w| w.id == workspace_id)
        .cloned();
    
    match workspace {
        Some(ws) => workspaces::delete_workspace(&profile_path, &[ws]).map_err(|e| e.to_string()),
        None => Err(format!("Workspace with ID {} not found", workspace_id))
    }
}

#[tauri::command]
async fn open_workspace(workspace_path: String, original_path: Option<String>) -> Result<bool, String> {
    // Use original_path if provided, otherwise fall back to workspace_path
    let path_to_open = original_path.unwrap_or(workspace_path);
    
    // Actually implement opening VSCode with the workspace
    #[cfg(target_os = "windows")]
    let code_command = "code";
    
    #[cfg(target_os = "macos")]
    let code_command = "code";
    
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let code_command = "code";
    
    match Command::new(code_command)
        .arg(path_to_open)
        .spawn() {
            Ok(_) => Ok(true),
            Err(e) => Err(e.to_string()),
        }
}

#[tauri::command]
async fn get_default_profile_path() -> Result<String, String> {
    workspaces::get_default_profile_path().map_err(|e| e.to_string())
}

#[tauri::command]
async fn workspace_exists(workspace: Workspace) -> Result<bool, String> {
    Ok(workspaces::workspace_exists(&workspace))
}

#[tauri::command]
fn get_known_vscode_paths() -> Result<Vec<String>, String> {
    Ok(get_known_vscode_paths_impl())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_workspaces,
            add_workspace,
            edit_workspace,
            delete_workspace,
            open_workspace,
            get_default_profile_path,
            workspace_exists,
            get_known_vscode_paths
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
} 