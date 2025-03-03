use crate::workspaces::{self, Workspace, workspace_exists};
use crate::tui::models::{InputMode, UiConfig};
use anyhow::Result;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Main application state
pub struct App {
    /// VSCode profile path
    pub profile_path: String,
    /// All available workspaces
    pub workspaces: Vec<Workspace>,
    /// Filtered workspaces (indices into workspaces)
    pub filtered_workspaces: Vec<usize>,
    /// Currently selected workspace index (in filtered_workspaces)
    pub selected_workspace_index: Option<usize>,
    /// Workspaces marked for deletion (by ID)
    pub marked_for_deletion: HashSet<String>,
    /// Current input mode
    pub input_mode: InputMode,
    /// Current input buffer for text input
    pub input_buffer: String,
    /// Cursor position in the input buffer
    pub cursor_position: usize,
    /// Current search query
    pub search_query: String,
    /// Status message to display
    pub status_message: Option<String>,
    /// Expiration time for the status message
    pub status_expiry: Option<Instant>,
    /// Current index in the autocomplete suggestions
    pub current_autocomplete_index: usize,
    /// Whether autocomplete is currently active
    pub is_autocomplete_active: bool,
    /// Current autocomplete suggestion (the full suggestion)
    pub autocomplete_suggestion: Option<String>,
    /// Position where the autocomplete suggestion starts
    pub autocomplete_start_position: usize,
    /// UI configuration settings
    pub ui_config: UiConfig,
    /// Known VSCode profile paths
    pub known_profile_paths: Vec<String>,
    /// Selected profile path index
    pub selected_profile_index: Option<usize>,
}

impl App {
    /// Create a new App instance with default values
    pub fn new(profile_path_arg: Option<&str>) -> Result<Self> {
        let profile_path = match profile_path_arg {
            Some(path) => path.to_string(),
            None => workspaces::get_default_profile_path()?
        };
        
        // Get known VSCode paths
        let known_profile_paths = workspaces::get_known_vscode_paths();
        
        Ok(Self {
            profile_path,
            workspaces: Vec::new(),
            filtered_workspaces: Vec::new(),
            selected_workspace_index: None,
            marked_for_deletion: HashSet::new(),
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            cursor_position: 0,
            search_query: String::new(),
            status_message: None,
            status_expiry: None,
            current_autocomplete_index: 0,
            is_autocomplete_active: false,
            autocomplete_suggestion: None,
            autocomplete_start_position: 0,
            ui_config: UiConfig::default(),
            known_profile_paths,
            selected_profile_index: None,
        })
    }

    /// Load workspaces from the profile
    pub fn load_workspaces(&mut self) -> Result<()> {
        self.workspaces = workspaces::get_workspaces(&self.profile_path)?;
        
        // Parse workspace paths to extract additional info
        for workspace in &mut self.workspaces {
            if workspace.parsed_info.is_none() {
                let _ = workspace.parse_path();
            }
        }
        
        self.apply_filter();
        if !self.filtered_workspaces.is_empty() && self.selected_workspace_index.is_none() {
            self.selected_workspace_index = Some(0);
        }
        Ok(())
    }

    /// Set a status message with an expiration time
    pub fn set_status(&mut self, message: &str, duration: Duration) {
        self.status_message = Some(message.to_string());
        self.status_expiry = Some(Instant::now() + duration);
    }

    /// Update and clear expired status messages
    pub fn update_status(&mut self) {
        if let Some(expiry) = self.status_expiry {
            if Instant::now() > expiry {
                self.status_message = None;
                self.status_expiry = None;
            }
        }
    }

    /// Apply the current search/filter to the workspaces
    pub fn apply_filter(&mut self) {
        let search_query = self.search_query.to_lowercase();
        let words: Vec<&str> = search_query.split_whitespace().collect();

        let mut filtered_workspaces = Vec::new();
        let mut remote_filter: Option<bool> = None;
        let mut type_filter: Option<&str> = None;
        let mut tag_filter: Option<&str> = None;
        let mut existence_filter: Option<bool> = None;
        let mut regular_keywords: Vec<&str> = Vec::new();

        for word in words {
            // Check for :remote: filter
            if word.starts_with(":remote:") {
                let value = word.trim_start_matches(":remote:");
                if value == "yes" {
                    remote_filter = Some(true);
                } else if value == "no" {
                    remote_filter = Some(false);
                }
            }
            // Check for :type: filter
            else if word.starts_with(":type:") {
                type_filter = Some(word.trim_start_matches(":type:"));
            }
            // Check for :tag: filter
            else if word.starts_with(":tag:") {
                tag_filter = Some(word.trim_start_matches(":tag:"));
            }
            // Check for :existing: filter
            else if word.starts_with(":existing:") {
                let value = word.trim_start_matches(":existing:");
                if value == "yes" {
                    existence_filter = Some(true);
                } else if value == "no" {
                    existence_filter = Some(false);
                }
            }
            // Regular keyword search
            else if !word.is_empty() {
                regular_keywords.push(word);
            }
        }

        // Apply filters to create indices of matching workspaces
        for (i, workspace) in self.workspaces.iter_mut().enumerate() {
            let mut include = true;

            // Remote filter
            if let Some(remote) = remote_filter {
                if workspace.is_remote() != remote {
                    include = false;
                }
            }

            // Type filter
            if include && type_filter.is_some() {
                let workspace_type = workspace.get_type();
                if let Some(filter_type) = type_filter {
                    match filter_type {
                        "folder" => {
                            if workspace_type != "folder" {
                                include = false;
                            }
                        }
                        "file" => {
                            if workspace_type != "file" {
                                include = false;
                            }
                        }
                        "workspace" => {
                            if workspace_type != "workspace" {
                                include = false;
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Tag filter
            if include && tag_filter.is_some() {
                if let Some(tag) = tag_filter {
                    let info_has_matching_tag = workspace.parse_path()
                        .map(|info| info.tags.iter().any(|t| t.to_lowercase().contains(tag)))
                        .unwrap_or(false);
                    
                    if !info_has_matching_tag {
                        include = false;
                    }
                }
            }

            // Existence filter
            if include && existence_filter.is_some() {
                if let Some(exists) = existence_filter {
                    let path_exists = workspace_exists(workspace);
                    if path_exists != exists {
                        include = false;
                    }
                }
            }

            // Regular keyword search
            if include && !regular_keywords.is_empty() {
                let label = workspace.get_label().to_lowercase();
                let path = workspace.path.to_lowercase();
                let tags = workspace.parse_path()
                    .map(|info| info.tags.join(" ").to_lowercase())
                    .unwrap_or_default();
                
                let combined_info = format!("{} {} {}", label, path, tags);
                
                if !regular_keywords.iter().all(|keyword| combined_info.contains(keyword)) {
                    include = false;
                }
            }

            if include {
                filtered_workspaces.push(i);
            }
        }

        self.filtered_workspaces = filtered_workspaces;
        self.selected_workspace_index = self.filtered_workspaces.first().map(|_| 0);
    }

    /// Toggle mark/unmark the currently selected workspace
    pub fn toggle_mark_selected(&mut self) {
        if let Some(selected_idx) = self.selected_workspace_index {
            if let Some(&workspace_idx) = self.filtered_workspaces.get(selected_idx) {
                if let Some(workspace) = self.workspaces.get(workspace_idx) {
                    let workspace_id = workspace.id.clone();
                    if self.marked_for_deletion.contains(&workspace_id) {
                        self.marked_for_deletion.remove(&workspace_id);
                    } else {
                        self.marked_for_deletion.insert(workspace_id);
                    }
                }
            }
        }
    }

    /// Delete all workspaces marked for deletion
    pub fn delete_marked_workspaces(&mut self) -> Result<()> {
        if self.marked_for_deletion.is_empty() {
            self.set_status("No workspaces marked for deletion", Duration::from_secs(2));
            return Ok(());
        }

        let total = self.marked_for_deletion.len();
        
        // Collect the workspaces to delete
        let workspaces_to_delete: Vec<Workspace> = self.workspaces.iter()
            .filter(|w| self.marked_for_deletion.contains(&w.id))
            .cloned()
            .collect();
            
        // Delete the workspaces
        let result = workspaces::delete_workspace(&self.profile_path, &workspaces_to_delete);
        
        // Clear the marked set
        self.marked_for_deletion.clear();
        
        // Reload workspaces to reflect changes
        self.load_workspaces()?;
        
        match result {
            Ok(true) => {
                self.set_status(
                    &format!("Successfully deleted {}/{} workspaces", workspaces_to_delete.len(), total),
                    Duration::from_secs(3),
                );
            },
            Ok(false) => {
                self.set_status(
                    "Some workspaces could not be deleted, check logs for details",
                    Duration::from_secs(3),
                );
            },
            Err(e) => {
                self.set_status(
                    &format!("Error deleting workspaces: {}", e),
                    Duration::from_secs(5),
                );
            }
        }
        
        Ok(())
    }

    /// Cancel the deletion of marked workspaces
    #[allow(dead_code)]
    pub fn cancel_deletion(&mut self) {
        self.marked_for_deletion.clear();
        self.set_status("Deletion canceled", Duration::from_secs(2));
    }

    /// Mark all filtered workspaces for deletion
    pub fn mark_all_filtered(&mut self) {
        let mut count = 0;
        for &workspace_idx in &self.filtered_workspaces {
            if let Some(workspace) = self.workspaces.get(workspace_idx) {
                self.marked_for_deletion.insert(workspace.id.clone());
                count += 1;
            }
        }
        
        if count > 0 {
            self.set_status(&format!("Marked {} workspaces for deletion", count), Duration::from_secs(2));
        }
    }
    
    /// Unmark all filtered workspaces
    pub fn unmark_all_filtered(&mut self) {
        let mut count = 0;
        for &workspace_idx in &self.filtered_workspaces {
            if let Some(workspace) = self.workspaces.get(workspace_idx) {
                if self.marked_for_deletion.remove(&workspace.id) {
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            self.set_status(&format!("Unmarked {} workspaces", count), Duration::from_secs(2));
        }
    }
    
    /// Toggle mark/unmark all filtered workspaces
    pub fn toggle_mark_all_filtered(&mut self) {
        let mut marked_count = 0;
        let mut unmarked_count = 0;
        
        // Individually toggle each workspace's selection state
        for &workspace_idx in &self.filtered_workspaces {
            if let Some(workspace) = self.workspaces.get(workspace_idx) {
                if self.marked_for_deletion.contains(&workspace.id) {
                    // If already marked, unmark it
                    self.marked_for_deletion.remove(&workspace.id);
                    unmarked_count += 1;
                } else {
                    // If not marked, mark it
                    self.marked_for_deletion.insert(workspace.id.clone());
                    marked_count += 1;
                }
            }
        }
        
        // Set status message with detailed counts
        if marked_count > 0 && unmarked_count > 0 {
            self.set_status(
                &format!("Toggled all: {} marked, {} unmarked", marked_count, unmarked_count),
                Duration::from_secs(2)
            );
        } else if marked_count > 0 {
            self.set_status(&format!("Marked {} workspaces", marked_count), Duration::from_secs(2));
        } else if unmarked_count > 0 {
            self.set_status(&format!("Unmarked {} workspaces", unmarked_count), Duration::from_secs(2));
        }
    }

    /// Get the current word at the cursor position, and the position of the start of the word
    pub fn get_current_word(&self) -> (&str, usize) {
        if self.input_buffer.is_empty() {
            return ("", 0);
        }

        // If autocomplete is active, we should return the original user input
        // not the expanded autocomplete suggestion
        if self.is_autocomplete_active && self.autocomplete_suggestion.is_some() {
            // Return only the user-typed part, before any autocomplete suggestion
            let before_cursor = &self.input_buffer[..self.cursor_position];
            
            // Find the start of the current word
            let word_start = before_cursor.rfind(' ').map_or(0, |pos| pos + 1);
            
            // Get what would be the user's input without autocomplete
            // This is the part from word_start to autocomplete_start_position
            if word_start <= self.autocomplete_start_position {
                return (&self.input_buffer[word_start..self.autocomplete_start_position], word_start);
            }
        }

        // Find word boundaries around the cursor
        let before_cursor = &self.input_buffer[..self.cursor_position];
        
        // Find the start of the current word (last space before cursor or start of string)
        let word_start = before_cursor.rfind(' ').map_or(0, |pos| pos + 1);
        
        // Return the current word up to the cursor
        (&self.input_buffer[word_start..self.cursor_position], word_start)
    }
} 