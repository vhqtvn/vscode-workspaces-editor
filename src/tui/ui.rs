use crate::tui::app::App;
use crate::tui::models::{InputMode, WorkspaceInfo};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Text, Line},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::workspaces;

/// Render the TUI interface
pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),    // Status line
                Constraint::Length(3),    // Input
                Constraint::Min(0),       // Main content area
                Constraint::Length(1),    // Help text
            ]
            .as_ref(),
        )
        .split(f.size());

    // Further split the main content area horizontally
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(70), // Workspace list
                Constraint::Percentage(30), // Details pane
            ]
            .as_ref(),
        )
        .split(chunks[2]);

    render_status_line(f, app, chunks[0]);
    render_input(f, app, chunks[1]);
    
    match app.input_mode {
        InputMode::SelectProfile => render_profile_selection(f, app, chunks[2]),
        _ => {
            render_workspaces(f, app, content_chunks[0]);
            render_details_pane(f, app, content_chunks[1]);
        }
    }
    
    render_help_text(f, app, chunks[3]);
}

/// Render the status line
fn render_status_line(f: &mut Frame, app: &App, area: Rect) {
    // Use a default message with the profile path when status is empty
    let status_text = match app.status_message.as_deref() {
        Some(msg) if !msg.is_empty() => msg.to_string(),
        _ => format!("VSCode WS Editor: {}", app.profile_path)
    };
    
    let status_style = if app.ui_config.use_colors {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    
    let status = Paragraph::new(status_text).style(status_style);
    f.render_widget(status, area);
}

/// Render the input area
fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let title;
    let delete_msg;
    let text;

    match app.input_mode {
        InputMode::Normal => {
            // Display "No Filter Applied" in the input field
            let style = if app.ui_config.use_colors {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };
            
            if app.search_query.is_empty() {
                text = Text::styled("No Filter Applied", style);
            } else {
                text = Text::styled(&app.search_query, style);
            }
            
            title = "Filter";
        },
        InputMode::ProfilePath => {
            text = Text::raw(&app.input_buffer);
            title = "Enter Profile Path";
        },
        InputMode::SelectProfile => {
            text = Text::raw("Select a VSCode profile or press 'c' to enter custom path");
            title = "Profile Selection";
        },
        InputMode::Searching => {
            // For searching mode, we need to handle autocomplete highlighting
            if app.is_autocomplete_active && app.autocomplete_suggestion.is_some() {
                let suggestion = app.autocomplete_suggestion.as_ref().unwrap();
                let start_pos = app.autocomplete_start_position;
                
                // Create styled text with underlined autocomplete suggestion
                let mut spans = Vec::new();
                
                // Add text before autocomplete suggestion
                if start_pos > 0 {
                    spans.push(Span::raw(&app.input_buffer[..start_pos]));
                }
                
                // Add underlined autocomplete suggestion
                let suggestion_style = Style::default().add_modifier(Modifier::UNDERLINED);
                spans.push(Span::styled(suggestion, suggestion_style));
                
                // Add any text after the autocomplete if needed
                let end_pos = start_pos + suggestion.len();
                if end_pos < app.input_buffer.len() {
                    spans.push(Span::raw(&app.input_buffer[end_pos..]));
                }
                
                text = Text::from(Line::from(spans));
            } else {
                text = Text::raw(&app.input_buffer);
            }
            title = "Filter";
        },
        InputMode::ConfirmDelete => {
            delete_msg = format!(
                "Delete {} marked workspace(s)? (y/n)",
                app.marked_for_deletion.len()
            );
            
            let style = if app.ui_config.use_colors {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };
            
            text = Text::styled(&delete_msg, style);
            title = "Confirm Deletion";
        }
    };

    let mut paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(title));

    // Set cursor position for input modes
    match app.input_mode {
        InputMode::ProfilePath | InputMode::Searching => {
            f.set_cursor(
                area.x + app.cursor_position as u16 + 1,
                area.y + 1,
            );
            paragraph = paragraph.style(if app.ui_config.use_colors {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });
        }
        _ => {}
    }

    f.render_widget(paragraph, area);
}

/// Render the workspaces list
fn render_workspaces(f: &mut Frame, app: &App, area: Rect) {
    // Calculate visible count and offset for scrolling
    let height = area.height as usize;
    let list_height = height.saturating_sub(2); // Subtract 2 for borders
    
    // For ConfirmDelete mode, filter the list to show only marked workspaces
    let visible_workspaces: Vec<usize> = app.filtered_workspaces.clone();
    
    // Create the list items
    let items: Vec<ListItem> = if visible_workspaces.is_empty() {
        // Show appropriate message based on whether there's a search filter
        let message = if !app.search_query.is_empty() {
            "No workspaces match your search criteria."
        } else {
            "No workspaces found in this VSCode profile."
        };
        
        vec![ListItem::new(message).style(
            if app.ui_config.use_colors {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            }
        )]
    } else {
        // We keep the selected_idx even in ConfirmDelete mode
        let selected_idx = app.selected_workspace_index;
        
        // Calculate offset for scrolling (keep selected item in view)
        let offset = if let Some(idx) = selected_idx {
            if idx >= list_height {
                idx - list_height + 1
            } else {
                0
            }
        } else {
            0
        };

        // Calculate the width of the list area (needed for full-width highlighting)
        let list_width = area.width.saturating_sub(2) as usize; // Subtract 2 for borders
        
        // Format items with style
        visible_workspaces
            .iter()
            .enumerate()
            .skip(offset)
            .take(list_height)
            .map(|(i, &workspace_idx)| {
                // Get the workspace
                if let Some(workspace) = app.workspaces.get(workspace_idx) {
                    // Check if this workspace is marked for deletion
                    let is_marked = app.marked_for_deletion.contains(&workspace.id);
                    
                    // Clone for methods that require mutability
                    let mut workspace_clone = workspace.clone();
                    
                    // Convert workspace to WorkspaceInfo for display
                    let workspace_info = WorkspaceInfo {
                        id: workspace.id.clone(),
                        name: workspace.name.clone(),
                        path: workspace.path.clone(),
                        exists: crate::workspaces::workspace_exists(workspace),
                        workspace_type: workspace_clone.get_type(),
                        is_remote: workspace_clone.is_remote(),
                        remote_user: workspace.parsed_info.as_ref()
                            .and_then(|info| info.remote_user.clone()),
                        remote_port: workspace.parsed_info.as_ref()
                            .and_then(|info| info.remote_port),
                        tags: workspace.parsed_info.as_ref()
                            .map(|info| info.tags.clone())
                            .unwrap_or_default(),
                    };
                    
                    // Format the workspace entry with style
                    let entry_spans = format_workspace_entry_styled(&workspace_info, is_marked, app);
                    
                    // Handle selection highlighting
                    let item_text = if let Some(selected_idx) = selected_idx {
                        if i == selected_idx {
                            // Get the content as a string to calculate width
                            let content = entry_spans.iter()
                                .map(|span| span.content.as_ref())
                                .collect::<String>();
                            
                            // Calculate content width and needed padding for full-width
                            let content_width = unicode_width::UnicodeWidthStr::width(content.as_str());
                            let padding_width = list_width.saturating_sub(content_width);
                            let padding = " ".repeat(padding_width);
                            
                            // Create a background color for highlighting
                            let highlight_bg = if app.ui_config.use_colors {
                                if is_marked { Color::Magenta } else { Color::Yellow }
                            } else {
                                Color::Reset // Not used in no-color mode
                            };
                            
                            // Create all spans with highlighting
                            let mut highlighted_spans: Vec<Span> = Vec::new();
                            
                            for span in entry_spans.iter() {
                                let style = if app.ui_config.use_colors {
                                    Style::default()
                                        .fg(Color::Black) // Black text for better contrast with yellow
                                        .bg(highlight_bg)
                                } else {
                                    Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
                                };
                                
                                highlighted_spans.push(Span::styled(span.content.clone(), style));
                            }
                            
                            // Add padding with the same background color
                            if padding_width > 0 {
                                let padding_style = if app.ui_config.use_colors {
                                    Style::default().bg(highlight_bg)
                                } else {
                                    Style::default().add_modifier(Modifier::REVERSED)
                                };
                                
                                highlighted_spans.push(Span::styled(padding, padding_style));
                            }
                            
                            Text::from(Line::from(highlighted_spans))
                        } else {
                            Text::from(Line::from(entry_spans.clone()))
                        }
                    } else {
                        Text::from(Line::from(entry_spans.clone()))
                    };
                    
                    ListItem::new(item_text)
                } else {
                    // Fallback for invalid workspace index
                    ListItem::new("Invalid workspace")
                }
            })
            .collect()
    };

    // Create the list widget
    let title = match app.input_mode {
        InputMode::ConfirmDelete => "Selected Workspaces to Delete",
        _ => "Workspaces",
    };
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

/// Format a workspace entry with color and style information
fn format_workspace_entry_styled(workspace: &WorkspaceInfo, is_marked: bool, app: &App) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    
    // Get whether to use colors or not
    let use_colors = app.ui_config.use_colors;
    
    // Add mark indicator
    let mark_style = if use_colors {
        if is_marked {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        }
    } else {
        Style::default()
    };
    
    spans.push(Span::styled(
        if is_marked { "[X] ".to_string() } else { "[ ] ".to_string() },
        mark_style
    ));
    
    // Add existence indicator
    let existence_style = if use_colors {
        if workspace.exists {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        }
    } else {
        Style::default()
    };
    
    spans.push(Span::styled(
        if workspace.exists { "✓ ".to_string() } else { "✗ ".to_string() },
        existence_style
    ));
    
    // Add type indicator with color
    let type_style = if use_colors {
        match workspace.workspace_type.as_str() {
            "folder" => Style::default().fg(Color::Blue),
            "workspace" => Style::default().fg(Color::Magenta),
            "file" => Style::default().fg(Color::Yellow),
            _ => Style::default().fg(Color::White),
        }
    } else {
        Style::default()
    };
    
    let type_icon = match workspace.workspace_type.as_str() {
        "folder" => "📁 ",
        "workspace" => "🔨 ",
        "file" => "📄 ",
        _ => "❓ ",
    };
    
    spans.push(Span::styled(
        type_icon.to_string(),
        type_style
    ));
    
    // Add remote indicator with color
    let remote_style = if use_colors {
        if workspace.is_remote {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Blue) // Changed from DarkGray to Blue
        }
    } else {
        Style::default()
    };
    
    spans.push(Span::styled(
        if workspace.is_remote { "🌐 ".to_string() } else { "🏠 ".to_string() },
        remote_style
    ));
    
    // Add name with appropriate style
    let name_style = if use_colors {
        if !workspace.exists {
            Style::default().fg(Color::Red) // Changed from DarkGray to Red
        } else {
            Style::default().fg(Color::White)
        }
    } else {
        Style::default()
    };
    
    // Extract name or use folder basename if unnamed
    let name = match &workspace.name {
        Some(name) if !name.is_empty() => name.clone(),
        _ => workspaces::extract_folder_basename(&workspace.path)
    };
    
    spans.push(Span::styled(
        name,
        name_style.add_modifier(Modifier::BOLD)
    ));
    
    // Add path with a dimmer style
    let path_style = if use_colors {
        Style::default().fg(Color::Blue) // Changed from DarkGray to Blue
    } else {
        Style::default()
    };
    
    spans.push(Span::styled(
        format!(" ({})", workspace.path),
        path_style
    ));
    
    spans
}

/// Format a workspace entry as plain string (used for simple display cases)
#[allow(dead_code)]
fn format_workspace_entry(workspace: &WorkspaceInfo, is_marked: bool) -> String {
    let mark = if is_marked { "[X] " } else { "[ ] " };
    
    // Extract name or use folder basename if unnamed
    let name = match &workspace.name {
        Some(name) if !name.is_empty() => name.clone(),
        _ => workspaces::extract_folder_basename(&workspace.path)
    };
    
    let existence_indicator = if workspace.exists {
        "●"
    } else {
        "○"
    };
    
    let ws_type = match workspace.workspace_type.as_str() {
        "file" => "F",
        "folder" => "D",
        "workspace" => "W",
        _ => "?",
    };
    
    let remote = if workspace.is_remote { "R" } else { "L" };
    
    format!(
        "{}{} {} {} {} {}",
        mark,
        existence_indicator,
        ws_type,
        remote,
        name,
        workspace.path
    )
}

/// Render details pane showing information about the selected workspace
fn render_details_pane(f: &mut Frame, app: &App, area: Rect) {
    let selected_workspace = app.selected_workspace_index
        .and_then(|i| app.filtered_workspaces.get(i))
        .map(|&idx| &app.workspaces[idx]);
    
    // Use brighter colors for the border to improve visibility
    let border_color = if app.ui_config.use_colors { Color::Cyan } else { Color::White };
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Details")
        .border_style(Style::default().fg(border_color));
    
    f.render_widget(block, area);
    
    // Return early if no workspace is selected
    let workspace = match selected_workspace {
        Some(w) => w,
        None => return,
    };
    
    // Clone to be able to call methods
    let mut workspace_clone = workspace.clone();
    
    // Create a smaller area for the content
    let content_area = Layout::default()
        .margin(1)
        .constraints([Constraint::Min(0)].as_ref())
        .split(area)[0];
    
    // Check if workspace exists
    let exists = crate::workspaces::workspace_exists(&workspace_clone);
    
    // Get workspace info
    let remote = workspace_clone.is_remote();
    let ws_type = workspace_clone.get_type();
    let tags = workspace_clone.parsed_info.as_ref()
        .map(|info| info.tags.join(", "))
        .unwrap_or_default();
    
    // Get remote user and port
    let remote_host = workspace_clone.parsed_info.as_ref()
        .and_then(|info| info.remote_host.clone());
    let remote_user = workspace_clone.parsed_info.as_ref()
        .and_then(|info| info.remote_user.clone());
    let remote_port = workspace_clone.parsed_info.as_ref()
        .and_then(|info| info.remote_port);
    
    // Format dates
    let last_used = if workspace.last_used > 0 {
        let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(workspace.last_used / 1000, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        dt
    } else {
        "Never".to_string()
    };
    
    // Create detail lines
    let mut detail_lines = vec![
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(if app.ui_config.use_colors { Color::Yellow } else { Color::White })),
            Span::raw({
                let name = match workspace.name.as_deref() {
                    Some(name) if !name.is_empty() => name.to_string(),
                    _ => workspaces::extract_folder_basename(&workspace.path),
                };
                name
            }),
        ]),
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(if app.ui_config.use_colors { Color::Yellow } else { Color::White })),
            Span::raw(&workspace.path),
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(if app.ui_config.use_colors { Color::Yellow } else { Color::White })),
            Span::styled(
                &ws_type, 
                Style::default().fg(if app.ui_config.use_colors {
                    match ws_type.as_str() {
                        "folder" => Color::Green,
                        "file" => Color::Blue,
                        "workspace" => Color::Magenta,
                        _ => Color::White,
                    }
                } else {
                    Color::White
                })
            ),
        ]),
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(if app.ui_config.use_colors { Color::Yellow } else { Color::White })),
            Span::styled(
                if exists { "Exists" } else { "Missing" },
                Style::default().fg(if app.ui_config.use_colors {
                    if exists { Color::Green } else { Color::Red }
                } else {
                    Color::White
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Remote: ", Style::default().fg(if app.ui_config.use_colors { Color::Yellow } else { Color::White })),
            Span::styled(
                if remote { "Yes" } else { "No" },
                Style::default().fg(if app.ui_config.use_colors {
                    if remote { Color::Cyan } else { Color::White }
                } else {
                    Color::White
                }),
            ),
        ]),
    ];
    
    // Add remote user and port information if available
    if remote {
        if let Some(host) = &remote_host {
            detail_lines.push(Line::from(vec![
                Span::styled("Host: ", Style::default().fg(if app.ui_config.use_colors { Color::Yellow } else { Color::White })),
                Span::styled(
                    host,
                    Style::default().fg(if app.ui_config.use_colors { Color::Cyan } else { Color::White }),
                ),
            ]));
        }
        
        if let Some(user) = &remote_user {
            detail_lines.push(Line::from(vec![
                Span::styled("User: ", Style::default().fg(if app.ui_config.use_colors { Color::Yellow } else { Color::White })),
                Span::styled(
                    user,
                    Style::default().fg(if app.ui_config.use_colors { Color::Cyan } else { Color::White }),
                ),
            ]));
        }
        
        if let Some(port) = remote_port {
            detail_lines.push(Line::from(vec![
                Span::styled("Port: ", Style::default().fg(if app.ui_config.use_colors { Color::Yellow } else { Color::White })),
                Span::styled(
                    port.to_string(),
                    Style::default().fg(if app.ui_config.use_colors { Color::Cyan } else { Color::White }),
                ),
            ]));
        }
    }
    
    // Add remaining details
    detail_lines.push(Line::from(vec![
        Span::styled("Last Used: ", Style::default().fg(if app.ui_config.use_colors { Color::Yellow } else { Color::White })),
        Span::raw(last_used),
    ]));
    
    detail_lines.push(Line::from(""));
    
    detail_lines.push(Line::from(vec![
        Span::styled("Tags: ", Style::default().fg(if app.ui_config.use_colors { Color::Yellow } else { Color::White })),
        Span::styled(
            if tags.is_empty() { "None" } else { &tags }, 
            Style::default().fg(if app.ui_config.use_colors { Color::Cyan } else { Color::White })
        ),
    ]));
    
    let detail_paragraph = Paragraph::new(Text::from(detail_lines))
        .wrap(ratatui::widgets::Wrap { trim: true });
    
    f.render_widget(detail_paragraph, content_area);
}

/// Render the profile selection list
fn render_profile_selection(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = if app.known_profile_paths.is_empty() {
        vec![ListItem::new("No VSCode profiles found. Press 'c' to enter a custom path.").style(
            if app.ui_config.use_colors {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            }
        )]
    } else {
        app.known_profile_paths
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let style = if Some(i) == app.selected_profile_index {
                    if app.ui_config.use_colors {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().add_modifier(Modifier::REVERSED)
                    }
                } else {
                    Style::default()
                };
                
                let exists = std::path::Path::new(path).exists();
                let indicator = if exists { "●" } else { "○" };
                
                let text = format!("{} {}", indicator, path);
                ListItem::new(text).style(style)
            })
            .collect()
    };

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("VSCode Profiles"));

    f.render_widget(list, area);
}

/// Render the help text
fn render_help_text(f: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.input_mode {
        InputMode::Normal => "q: quit, p: set profile, f/: search, r: reload, Enter: toggle item, Ctrl+Alt+A: select/deselect all, Ctrl+Alt+T: toggle each item, d: delete, ↑/↓: navigate",
        InputMode::ProfilePath => "Enter: save, Esc: cancel",
        InputMode::SelectProfile => "Enter: select profile, c: enter custom path, ↑/↓: navigate, Esc: cancel",
        InputMode::Searching => "Enter: toggle item, Tab: autocomplete, Ctrl+Alt+A: select/deselect all, Ctrl+Alt+T: toggle each item, ↑/↓: navigate, Esc: exit search, Filters: :existing:yes/no, :type:, :remote:yes/no, :tag:",
        InputMode::ConfirmDelete => "y: confirm, n/Esc: cancel, ↑/↓: navigate through selected workspaces, Enter: unmark selected workspace",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(if app.ui_config.use_colors { Color::Yellow } else { Color::White }));
    f.render_widget(help, area);
} 