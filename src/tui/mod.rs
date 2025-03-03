mod app;
mod models;
mod ui;
mod input_handler;
mod autocomplete;

use std::io;
use std::time::{Duration, Instant};
use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{CrosstermBackend},
    Terminal,
};

pub use app::App;

/// Run the TUI application
pub fn run(profile_path: Option<&str>) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(profile_path)?;
    
    // Load workspaces on startup
    app.load_workspaces()?;

    // Set status message
    app.set_status(
        &format!("Loaded {} workspaces", app.workspaces.len()),
        Duration::from_secs(3),
    );

    // Main event loop
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // Draw the UI
        terminal.draw(|f| ui::render(f, &app))?;

        // Handle events
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Handle key events for the current mode
                if input_handler::handle_key_event(&mut app, key)? {
                    break;
                }
            }
        }
        
        // Tick update
        if last_tick.elapsed() >= tick_rate {
            app.update_status();
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;

    Ok(())
} 