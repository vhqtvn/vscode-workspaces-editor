use crate::tui::app::App;
use crate::tui::autocomplete;
use crate::tui::models::InputMode;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Handle keyboard events in the TUI
pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<bool> {
    // Special case for Ctrl+C in any mode
    if let KeyCode::Char('c') = key.code {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(true); // Signal to quit
        }
    }

    match app.input_mode {
        InputMode::Normal => handle_normal_mode(app, key),
        InputMode::ProfilePath => handle_profile_path_mode(app, key),
        InputMode::SelectProfile => handle_select_profile_mode(app, key),
        InputMode::Searching => handle_search_mode(app, key),
        InputMode::ConfirmDelete => handle_confirm_delete_mode(app, key),
    }
}

/// Handle keyboard events in normal mode
fn handle_normal_mode(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('q') => Ok(true), // quit
        KeyCode::Char('r') => {
            app.load_workspaces().unwrap_or_else(|e| {
                app.set_status(&format!("Error: {}", e), Duration::from_secs(5));
            });
            app.set_status("Workspaces reloaded", Duration::from_secs(2));
            Ok(false)
        }
        KeyCode::Char('p') => {
            app.input_mode = InputMode::SelectProfile;
            app.selected_profile_index = app.known_profile_paths
                .iter()
                .position(|p| p == &app.profile_path);
            app.set_status("Select VSCode profile or press 'c' to enter custom path", Duration::from_secs(3));
            Ok(false)
        }
        KeyCode::Char('f') | KeyCode::Char('/') => {
            app.input_mode = InputMode::Searching;
            app.input_buffer = app.search_query.clone();
            app.cursor_position = app.input_buffer.len();
            Ok(false)
        }
        // Enter: Toggle mark/unmark for selected item
        KeyCode::Enter => {
            app.toggle_mark_selected();
            app.set_status("Toggled current workspace", Duration::from_secs(1));
            Ok(false)
        }
        // Ctrl+Alt+A: Select/deselect all items in filtered view
        KeyCode::Char('a')
            if key
                .modifiers
                .contains(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            // Check if all filtered workspaces are already marked
            let all_marked = app.filtered_workspaces.iter().all(|&idx| {
                if let Some(workspace) = app.workspaces.get(idx) {
                    app.marked_for_deletion.contains(&workspace.id)
                } else {
                    false
                }
            });

            if all_marked {
                app.unmark_all_filtered();
                app.set_status(
                    "Deselected all workspaces in filtered view",
                    Duration::from_secs(2),
                );
            } else {
                app.mark_all_filtered();
                app.set_status(
                    "Selected all workspaces in filtered view",
                    Duration::from_secs(2),
                );
            }
            Ok(false)
        }
        // Ctrl+Alt+T: Toggle selection state for all items in filtered view
        KeyCode::Char('t')
            if key
                .modifiers
                .contains(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            app.toggle_mark_all_filtered();
            app.set_status(
                "Toggled all workspaces individually",
                Duration::from_secs(2),
            );
            Ok(false)
        }
        KeyCode::Char('d') => {
            if !app.marked_for_deletion.is_empty() {
                app.filtered_workspaces = app
                    .marked_for_deletion
                    .iter()
                    .map(|id| app.workspaces.iter().position(|w| w.id == *id).unwrap())
                    .collect();
                app.input_mode = InputMode::ConfirmDelete;
            } else {
                app.set_status("No workspaces marked for deletion", Duration::from_secs(2));
            }
            Ok(false)
        }
        KeyCode::Up => {
            if let Some(index) = app.selected_workspace_index {
                if index > 0 {
                    app.selected_workspace_index = Some(index - 1);
                }
            }
            Ok(false)
        }
        KeyCode::Down => {
            if let Some(index) = app.selected_workspace_index {
                if index < app.filtered_workspaces.len() - 1 {
                    app.selected_workspace_index = Some(index + 1);
                }
            } else if !app.filtered_workspaces.is_empty() {
                app.selected_workspace_index = Some(0);
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle keyboard events in profile path editing mode
fn handle_profile_path_mode(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Enter => {
            app.profile_path = app.input_buffer.clone();
            app.input_mode = InputMode::Normal;
            app.load_workspaces().unwrap_or_else(|e| {
                app.set_status(&format!("Error: {}", e), Duration::from_secs(5));
            });
            Ok(false)
        }
        KeyCode::Char(c) => {
            app.input_buffer.insert(app.cursor_position, c);
            app.cursor_position += 1;
            Ok(false)
        }
        KeyCode::Backspace => {
            if app.cursor_position > 0 {
                app.input_buffer.remove(app.cursor_position - 1);
                app.cursor_position -= 1;
            }
            Ok(false)
        }
        KeyCode::Left => {
            if app.cursor_position > 0 {
                app.cursor_position -= 1;
            }
            Ok(false)
        }
        KeyCode::Right => {
            if app.cursor_position < app.input_buffer.len() {
                app.cursor_position += 1;
            }
            Ok(false)
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle keyboard events in profile selection mode
fn handle_select_profile_mode(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Enter => {
            if let Some(index) = app.selected_profile_index {
                if let Some(path) = app.known_profile_paths.get(index) {
                    app.profile_path = path.clone();
                    app.input_mode = InputMode::Normal;
                    app.load_workspaces().unwrap_or_else(|e| {
                        app.set_status(&format!("Error: {}", e), Duration::from_secs(5));
                    });
                }
            }
            Ok(false)
        }
        KeyCode::Char('c') => {
            app.input_mode = InputMode::ProfilePath;
            app.input_buffer = app.profile_path.clone();
            app.cursor_position = app.input_buffer.len();
            Ok(false)
        }
        KeyCode::Up => {
            if let Some(index) = app.selected_profile_index {
                if index > 0 {
                    app.selected_profile_index = Some(index - 1);
                }
            } else if !app.known_profile_paths.is_empty() {
                app.selected_profile_index = Some(app.known_profile_paths.len() - 1);
            }
            Ok(false)
        }
        KeyCode::Down => {
            if let Some(index) = app.selected_profile_index {
                if index < app.known_profile_paths.len() - 1 {
                    app.selected_profile_index = Some(index + 1);
                }
            } else if !app.known_profile_paths.is_empty() {
                app.selected_profile_index = Some(0);
            }
            Ok(false)
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle keyboard events in search mode
fn handle_search_mode(app: &mut App, key: KeyEvent) -> Result<bool> {
    // First check if autocomplete is active and this is not a Tab key
    // If so, commit the autocomplete before continuing with normal key handling
    if app.is_autocomplete_active && key.code != KeyCode::Tab {
        autocomplete::commit_autocomplete(app);
    }

    match key.code {
        KeyCode::Enter => {
            // Toggle the selected item
            app.toggle_mark_selected();
            app.set_status("Toggled current workspace", Duration::from_secs(1));
            Ok(false)
        }
        KeyCode::Backspace => {
            if app.cursor_position > 0 {
                app.input_buffer.remove(app.cursor_position - 1);
                app.cursor_position -= 1;

                // Reset autocomplete index when text changes
                app.current_autocomplete_index = 0;
                app.is_autocomplete_active = false;

                update_search_results(app);
            }
            Ok(false)
        }
        KeyCode::Left => {
            if app.cursor_position > 0 {
                app.cursor_position -= 1;
            }
            Ok(false)
        }
        KeyCode::Right => {
            if app.cursor_position < app.input_buffer.len() {
                app.cursor_position += 1;
            }
            Ok(false)
        }
        KeyCode::Up => {
            if let Some(index) = app.selected_workspace_index {
                if index > 0 {
                    app.selected_workspace_index = Some(index - 1);
                }
            } else if !app.filtered_workspaces.is_empty() {
                app.selected_workspace_index = Some(0);
            }
            Ok(false)
        }
        KeyCode::Down => {
            if let Some(index) = app.selected_workspace_index {
                if index < app.filtered_workspaces.len() - 1 {
                    app.selected_workspace_index = Some(index + 1);
                }
            } else if !app.filtered_workspaces.is_empty() {
                app.selected_workspace_index = Some(0);
            }
            Ok(false)
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;

            // Reset the autocomplete index when exiting search mode
            app.current_autocomplete_index = 0;
            app.is_autocomplete_active = false;

            if !app.search_query.is_empty() {
                app.search_query = String::new();
                app.apply_filter();
                app.set_status("Search cleared", Duration::from_secs(1));
            }
            Ok(false)
        }
        // Ctrl+Alt+A: Select/deselect all items in filtered view
        KeyCode::Char('a')
            if key
                .modifiers
                .contains(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            // Check if all filtered workspaces are already marked
            let all_marked = app.filtered_workspaces.iter().all(|&idx| {
                if let Some(workspace) = app.workspaces.get(idx) {
                    app.marked_for_deletion.contains(&workspace.id)
                } else {
                    false
                }
            });

            if all_marked {
                app.unmark_all_filtered();
                app.set_status(
                    "Deselected all workspaces in filtered view",
                    Duration::from_secs(2),
                );
            } else {
                app.mark_all_filtered();
                app.set_status(
                    "Selected all workspaces in filtered view",
                    Duration::from_secs(2),
                );
            }
            Ok(false)
        }
        // Ctrl+Alt+T: Toggle selection state for all items in filtered view
        KeyCode::Char('t')
            if key
                .modifiers
                .contains(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            app.toggle_mark_all_filtered();
            app.set_status(
                "Toggled all workspaces individually",
                Duration::from_secs(2),
            );
            Ok(false)
        }
        KeyCode::Tab => {
            autocomplete::process_tab_key(app);
            Ok(false)
        }
        KeyCode::Char(c) => {
            app.input_buffer.insert(app.cursor_position, c);
            app.cursor_position += 1;

            // Reset autocomplete index when text changes
            app.current_autocomplete_index = 0;
            app.is_autocomplete_active = false;

            update_search_results(app);
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle keyboard events in confirm delete mode
fn handle_confirm_delete_mode(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('y') => {
            if let Err(e) = app.delete_marked_workspaces() {
                app.set_status(&format!("Error: {}", e), Duration::from_secs(5));
            }
            app.input_mode = InputMode::Normal;
            Ok(false)
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.set_status("Deletion cancelled", Duration::from_secs(2));
            app.apply_filter();
            app.selected_workspace_index = None;
            Ok(false)
        }
        KeyCode::Enter => {
            // Allow unmarking workspaces from the confirmation screen
            if let Some(selected_idx) = app.selected_workspace_index {
                let marked_indices: Vec<usize> = app
                    .filtered_workspaces
                    .iter()
                    .enumerate()
                    .filter(|(_, &idx)| {
                        if let Some(workspace) = app.workspaces.get(idx) {
                            app.marked_for_deletion.contains(&workspace.id)
                        } else {
                            false
                        }
                    })
                    .map(|(i, _)| i)
                    .collect();

                if !marked_indices.is_empty() {
                    // Make sure the selected index is within the filtered view
                    if selected_idx < app.filtered_workspaces.len() {
                        let workspace_idx = app.filtered_workspaces[selected_idx];
                        if let Some(workspace) = app.workspaces.get(workspace_idx) {
                            if app.marked_for_deletion.contains(&workspace.id) {
                                app.marked_for_deletion.remove(&workspace.id);
                                app.set_status(
                                    "Removed workspace from selection",
                                    Duration::from_secs(1),
                                );

                                // If no more workspaces are marked, exit confirm mode
                                if app.marked_for_deletion.is_empty() {
                                    app.input_mode = InputMode::Normal;
                                    app.set_status(
                                        "No workspaces marked for deletion",
                                        Duration::from_secs(2),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Ok(false)
        }
        KeyCode::Up => {
            if let Some(selected) = app.selected_workspace_index {
                if selected > 0 {
                    app.selected_workspace_index = Some(selected - 1);
                }
            } else if !app.filtered_workspaces.is_empty() {
                app.selected_workspace_index = Some(0);
            }
            Ok(false)
        }
        KeyCode::Down => {
            if let Some(selected) = app.selected_workspace_index {
                if selected < app.filtered_workspaces.len() - 1 {
                    app.selected_workspace_index = Some(selected + 1);
                }
            } else if !app.filtered_workspaces.is_empty() {
                app.selected_workspace_index = Some(0);
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Update search results and display count
fn update_search_results(app: &mut App) {
    app.search_query = app.input_buffer.clone();
    app.apply_filter();

    let count = app.filtered_workspaces.len();
    if count == 0 {
        app.set_status("No matches found", Duration::from_secs(1));
    } else {
        app.set_status(&format!("Found {} matches", count), Duration::from_secs(1));
    }
}
