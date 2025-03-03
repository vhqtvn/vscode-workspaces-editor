use crate::tui::app::App;
use std::time::Duration;

/// Available filter modifiers
pub const FILTER_MODIFIERS: [&str; 5] = [":existing:", ":remote:", ":type:", ":path:", ":tag:"];

/// Available values for the :existing: filter
pub const EXISTING_VALUES: [&str; 2] = ["yes", "no"];

/// Available values for the :remote: filter
pub const REMOTE_VALUES: [&str; 2] = ["yes", "no"];

/// Available values for the :type: filter
pub const TYPE_VALUES: [&str; 3] = ["folder", "file", "workspace"];

/// Process Tab key press for autocomplete
pub fn process_tab_key(app: &mut App) {
    let (current_word, position_before_word) = app.get_current_word();
    let current_word = current_word.to_string();

    // Check if we're trying to autocomplete a modifier's value
    // Use current_word to check if it's a complete modifier
    let modifier_context = FILTER_MODIFIERS
        .iter()
        .find(|&modifier| &current_word == modifier)
        .copied();

        // Now handle the different autocomplete scenarios
    if let Some(modifier) = modifier_context {
        // Autocomplete for modifier values
        process_value_autocomplete(app, modifier);
    } else if current_word.starts_with(":") {
        // Check if we already have a complete modifier first
        if current_word.ends_with(":") && FILTER_MODIFIERS.contains(&current_word.as_str()) {
            // Already have a complete modifier, show value options
            app.autocomplete_suggestion = Some(current_word.to_string());
            app.autocomplete_start_position = app.cursor_position - current_word.len();
            show_filter_help(app, &current_word);
            app.is_autocomplete_active = true;
            return;
        }

        let matches: Vec<&str> = FILTER_MODIFIERS
            .iter()
            .filter_map(|m| m.strip_prefix(&current_word))
            .collect();

        if !matches.is_empty() {
            // If we have multiple matches and autocomplete is active, cycle through them
            if matches.len() > 1 && app.is_autocomplete_active {
                // Increment the index or wrap around
                app.current_autocomplete_index =
                    (app.current_autocomplete_index + 1) % matches.len();
            } else {
                // First Tab press or only one match
                app.current_autocomplete_index = 0;
                app.is_autocomplete_active = true;
            }

            // Get the current match based on the index
            let current_match = matches[app.current_autocomplete_index];

            // Store autocomplete information (preserve the original user input)
            app.autocomplete_suggestion = Some(current_match.to_string());
            app.autocomplete_start_position = position_before_word + current_word.len();

            // Replace the partial input with the full modifier
            app.input_buffer.replace_range(
                app.autocomplete_start_position..app.cursor_position,
                current_match,
            );
            app.cursor_position = app.autocomplete_start_position + current_match.len();

            // Show a status message
            if matches.len() > 1 {
                app.set_status(
                    &format!(
                        "Selected {} ({}/{}) - Press Tab again to cycle",
                        current_match,
                        app.current_autocomplete_index + 1,
                        matches.len()
                    ),
                    Duration::from_secs(3),
                );
            } else {
                show_filter_help(app, current_match);
            }
        } else {
            app.set_status("No matching filter found", Duration::from_secs(2));
            app.is_autocomplete_active = false;
            app.autocomplete_suggestion = None;
        }
    } else {
        app.is_autocomplete_active = false;
        app.autocomplete_suggestion = None;
    }

    // Live update of search results
    app.search_query = app.input_buffer.clone();
    app.apply_filter();
}

/// Process autocomplete for modifier values
fn process_value_autocomplete(app: &mut App, modifier: &str) {
    // Determine which value set to use
    let values = match modifier {
        ":existing:" => &EXISTING_VALUES[..],
        ":remote:" => &REMOTE_VALUES[..],
        ":type:" => &TYPE_VALUES[..],
        ":path:" | ":tag:" => {
            // These don't have predetermined values
            app.set_status(
                &format!("Type a value for {}", modifier),
                Duration::from_secs(2),
            );
            return;
        }
        _ => return,
    };

    // Extract necessary information before mutating
    let (value_start_pos, current_value) = {
        let before_cursor = &app.input_buffer[..app.cursor_position];
        let modifier_pos = before_cursor.rfind(modifier).unwrap();
        let value_start = modifier_pos + modifier.len();
        let current = before_cursor[value_start..].to_string();
        (value_start, current)
    };

    // If there's no user input yet, or if autocomplete is already active and
    // the user presses Tab again, cycle through all values
    if current_value.is_empty() || app.is_autocomplete_active {
        let next_index = if app.is_autocomplete_active {
            // Already in autocomplete mode, get next value
            (app.current_autocomplete_index + 1) % values.len()
        } else {
            // First Tab press, start with first value
            0
        };

        // Store autocomplete information
        app.autocomplete_suggestion = Some(values[next_index].to_string());
        app.autocomplete_start_position = value_start_pos;

        // Replace with the selected value
        app.input_buffer
            .replace_range(value_start_pos..app.cursor_position, values[next_index]);
        app.cursor_position = value_start_pos + values[next_index].len();
        app.current_autocomplete_index = next_index;
        app.is_autocomplete_active = true;

        // Show status message
        if values.len() > 1 {
            app.set_status(
                &format!(
                    "Selected {} value: {} ({}/{})",
                    modifier,
                    values[next_index],
                    next_index + 1,
                    values.len()
                ),
                Duration::from_secs(2),
            );
        } else {
            app.set_status(
                &format!("Selected {} value: {}", modifier, values[next_index]),
                Duration::from_secs(2),
            );
        }
    } else {
        // User has started typing a value, try to match it
        let matches: Vec<&str> = values
            .iter()
            .filter(|v| v.starts_with(&current_value))
            .copied()
            .collect();

        if matches.is_empty() {
            // No matches for what user typed, start cycling from beginning
            app.autocomplete_suggestion = Some(values[0].to_string());
            app.autocomplete_start_position = value_start_pos;

            // Replace with the first value
            app.input_buffer
                .replace_range(value_start_pos..app.cursor_position, values[0]);
            app.cursor_position = value_start_pos + values[0].len();
            app.current_autocomplete_index = 0;
            app.is_autocomplete_active = true;

            app.set_status(
                &format!("No matches. Selected {} value: {}", modifier, values[0]),
                Duration::from_secs(2),
            );
        } else if matches.len() == 1 {
            // Only one match, use it
            app.autocomplete_suggestion = Some(matches[0].to_string());
            app.autocomplete_start_position = value_start_pos;

            // Replace with the matching value
            app.input_buffer
                .replace_range(value_start_pos..app.cursor_position, matches[0]);
            app.cursor_position = value_start_pos + matches[0].len();
            app.current_autocomplete_index = 0;
            app.is_autocomplete_active = true;

            app.set_status(
                &format!("Selected {} value: {}", modifier, matches[0]),
                Duration::from_secs(2),
            );
        } else {
            // Multiple matches, cycle through them
            if app.is_autocomplete_active {
                // Cycle to next match
                app.current_autocomplete_index =
                    (app.current_autocomplete_index + 1) % matches.len();
            } else {
                // First tab press, select first match
                app.current_autocomplete_index = 0;
                app.is_autocomplete_active = true;
            }

            let current_match = matches[app.current_autocomplete_index];
            app.autocomplete_suggestion = Some(current_match.to_string());
            app.autocomplete_start_position = value_start_pos;

            // Replace with selected match
            app.input_buffer
                .replace_range(value_start_pos..app.cursor_position, current_match);
            app.cursor_position = value_start_pos + current_match.len();

            app.set_status(
                &format!(
                    "Selected {} value: {} ({}/{})",
                    modifier,
                    current_match,
                    app.current_autocomplete_index + 1,
                    matches.len()
                ),
                Duration::from_secs(2),
            );
        }
    }

    // Live update search results
    app.search_query = app.input_buffer.clone();
    app.apply_filter();
}

/// Commit the current autocomplete selection
pub fn commit_autocomplete(app: &mut App) {
    // Mark autocomplete as no longer active and clear suggestion
    app.is_autocomplete_active = false;
    app.autocomplete_suggestion = None;
    app.current_autocomplete_index = 0;
}

/// Show help text for the selected filter
fn show_filter_help(app: &mut App, filter: &str) {
    match filter {
        ":existing:" => {
            app.set_status(
                "Filter values for :existing: - yes, no",
                Duration::from_secs(3),
            );
        }
        ":remote:" => {
            app.set_status(
                "Filter values for :remote: - yes, no",
                Duration::from_secs(3),
            );
        }
        ":type:" => {
            app.set_status(
                "Filter values for :type: - folder, file, workspace",
                Duration::from_secs(3),
            );
        }
        ":path:" => {
            app.set_status("Filter by path - :path:value", Duration::from_secs(3));
        }
        ":tag:" => {
            app.set_status("Filter by tag - :tag:value", Duration::from_secs(3));
        }
        _ => {
            app.set_status(
                &format!("Type a value for {}", filter),
                Duration::from_secs(2),
            );
        }
    }
}
