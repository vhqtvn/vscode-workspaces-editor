mod workspaces;
mod tui;
mod cli;

use clap::{Parser, Subcommand};
use anyhow::Result;

/// VSCode Workspaces Editor
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Path to the workspaces storage profile (if not provided, default profile will be used)
    #[clap(short, long)]
    profile: Option<String>,
    
    /// Disable colored output (alternatively, set NO_COLOR environment variable)
    #[clap(long)]
    no_color: bool,

    /// CLI Subcommands
    #[clap(subcommand)]
    command: Option<Commands>,
}

/// Available CLI subcommands
#[derive(Subcommand, Debug)]
enum Commands {
    /// List all workspaces
    List {
        /// Output format (text or json)
        #[clap(short, long, default_value = "text")]
        format: String,
    },
    /// Parse a specific workspace path (for testing)
    Parse {
        /// The workspace path to parse
        path: String,
    },
    /// Diagnose a specific workspace by ID or path
    Diagnose {
        /// The workspace ID or full path to diagnose
        #[clap(name = "id-or-path")]
        id_or_path: String,
        
        /// Profile path (uses default if not specified)
        #[clap(short, long)]
        profile: Option<String>,
    },
    /// Open a workspace with VSCode
    Open {
        /// The workspace ID or full path to open
        #[clap(name = "id-or-path")]
        id_or_path: String,
        
        /// Profile path (uses default if not specified)
        #[clap(short, long)]
        profile: Option<String>,
        
        /// Use parsed path instead of original path
        #[clap(long)]
        use_parsed: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();
    
    // Parse command line arguments
    let args = Args::parse();
    
    // Set NO_COLOR environment variable if --no-color flag is used
    if args.no_color {
        std::env::set_var("NO_COLOR", "1");
    }

    // Handle subcommands if present
    if let Some(cmd) = &args.command {
        match cmd {
            Commands::List { format } => {
                // Get profile path (default or user-provided)
                let profile_path = match &args.profile {
                    Some(path) => path.clone(),
                    None => workspaces::get_default_profile_path()?,
                };
                
                // Load workspaces
                let mut workspaces = workspaces::get_workspaces(&profile_path)?;
                
                // Parse workspace paths for all workspaces
                for workspace in &mut workspaces {
                    let _ = workspace.parse_path();
                }
                
                // Output the list
                cli::list_workspaces(&workspaces, format)?;
                return Ok(());
            },
            Commands::Parse { path } => {
                // Parse the given workspace path
                println!("Parsing workspace path: {}", path);
                match workspaces::parser::parse_workspace_path(path) {
                    Ok(info) => {
                        println!("Successfully parsed workspace path!");
                        println!("Type: {:?}", info.workspace_type);
                        println!("Remote Authority: {:?}", info.remote_authority);
                        println!("Remote Host: {:?}", info.remote_host);
                        println!("Path: {}", info.path);
                        if let Some(container) = info.container_path {
                            println!("Container Path: {}", container);
                        }
                        if !info.tags.is_empty() {
                            println!("Tags: {}", info.tags.join(", "));
                        }
                    },
                    Err(e) => {
                        println!("Failed to parse workspace path: {}", e);
                    }
                }
                return Ok(());
            },
            Commands::Diagnose { id_or_path, profile } => {
                // Get profile path (default or user-provided)
                let profile_path = match profile {
                    Some(path) => path.clone(),
                    None => match &args.profile {
                        Some(path) => path.clone(),
                        None => workspaces::get_default_profile_path()?,
                    },
                };
                
                println!("Diagnosing workspace with profile: {}", profile_path);
                println!("Looking for workspace by ID or path: {}", id_or_path);
                
                // Load workspaces
                let mut workspaces = workspaces::get_workspaces(&profile_path)?;
                
                // Try to find the workspace by ID or path
                let id_or_path_str = id_or_path.as_str();
                let matching_workspace = workspaces.iter_mut().find(|ws| 
                    ws.id == id_or_path_str || ws.path == id_or_path_str
                );
                
                if let Some(workspace) = matching_workspace {
                    println!("\nFound workspace:");
                    println!("ID: {}", workspace.id);
                    println!("Path: {}", workspace.path);
                    if let Some(name) = &workspace.name {
                        println!("Name: {}", name);
                    }
                    
                    println!("\nParsing workspace path...");
                    match workspace.parse_path() {
                        Some(info) => {
                            println!("Successfully parsed workspace path!");
                            println!("Type: {:?}", info.workspace_type);
                            if let Some(auth) = &info.remote_authority {
                                println!("Remote Authority: {}", auth);
                            }
                            if let Some(host) = &info.remote_host {
                                println!("Remote Host: {}", host);
                            }
                            println!("Path: {}", info.path);
                            if let Some(container) = &info.container_path {
                                println!("Container Path: {}", container);
                            }
                            if !info.tags.is_empty() {
                                println!("Tags: {}", info.tags.join(", "));
                            }
                        },
                        None => {
                            println!("Failed to parse workspace path!");
                        }
                    }
                    
                    // Show sources
                    println!("\nSources:");
                    for source in &workspace.sources {
                        match source {
                            workspaces::WorkspaceSource::Storage(path) =>
                                println!("Storage: {}", path),
                            workspaces::WorkspaceSource::Database(key) =>
                                println!("Database: {}", key),
                            workspaces::WorkspaceSource::Zed(channel) =>
                                println!("Zed({})", channel),
                        }
                    }
                } else {
                    println!("No workspace found with the given ID or path.");
                    
                    // Try to parse it as a path anyway
                    println!("\nTrying to parse as workspace path...");
                    match workspaces::parser::parse_workspace_path(id_or_path) {
                        Ok(info) => {
                            println!("Successfully parsed as a workspace path!");
                            println!("Type: {:?}", info.workspace_type);
                            if let Some(auth) = info.remote_authority {
                                println!("Remote Authority: {}", auth);
                            }
                            if let Some(host) = info.remote_host {
                                println!("Remote Host: {}", host);
                            }
                            println!("Path: {}", info.path);
                            if let Some(container) = info.container_path {
                                println!("Container Path: {}", container);
                            }
                            if !info.tags.is_empty() {
                                println!("Tags: {}", info.tags.join(", "));
                            }
                        },
                        Err(e) => {
                            println!("Failed to parse as workspace path: {}", e);
                        }
                    }
                }
                
                return Ok(());
            },
            Commands::Open { id_or_path, profile, use_parsed } => {
                // Get profile path (default or user-provided)
                let profile_path = match profile {
                    Some(path) => path.clone(),
                    None => match &args.profile {
                        Some(path) => path.clone(),
                        None => workspaces::get_default_profile_path()?,
                    },
                };
                
                // Load workspaces
                let mut workspaces = workspaces::get_workspaces(&profile_path)?;
                
                // Try to find the workspace by ID or path
                let id_or_path_str = id_or_path.as_str();
                let matching_workspace = workspaces.iter_mut().find(|ws| 
                    ws.id == id_or_path_str || ws.path == id_or_path_str
                );
                
                if let Some(workspace) = matching_workspace {
                    println!("Found workspace: {} ({})", 
                        workspace.name.as_deref().unwrap_or(&workspace.id), 
                        workspace.path
                    );
                    
                    // Parse the workspace path to get the original path
                    let parsed_info = workspace.parse_path();
                    
                    if let Some(info) = parsed_info {
                        // Determine which path to use
                        let path_to_use = if *use_parsed {
                            &workspace.path
                        } else {
                            &info.original_path
                        };
                        
                        println!("Opening workspace with {}path: {}", 
                            if *use_parsed { "parsed " } else { "original " },
                            path_to_use
                        );
                        
                        // Open the workspace
                        cli::open_workspace(path_to_use)?;
                    } else {
                        println!("Failed to parse workspace path. Using provided path.");
                        cli::open_workspace(&workspace.path)?;
                    }
                } else {
                    // If not found in stored workspaces, try to use the path directly
                    println!("No workspace found with ID/path: {}. Trying to open directly.", id_or_path);
                    cli::open_workspace(id_or_path)?;
                }
                
                return Ok(());
            }
        }
    }
    
    tui::run(args.profile.as_deref())?;
    
    Ok(())
}
