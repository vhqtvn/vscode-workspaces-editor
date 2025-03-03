use std::fs;
use std::path::Path;

fn main() {
    // Create icon copies if not already present
    create_icon_copies().expect("Failed to create icon copies");
    
    // Build the Tauri app
    tauri_build::build();
}

fn create_icon_copies() -> Result<(), std::io::Error> {
    let icons_dir = Path::new("icons");
    if !icons_dir.exists() {
        fs::create_dir_all(icons_dir)?;
    }
    
    let icon_path = icons_dir.join("icon.png");
    
    // Create copies for the other required icon files
    for name in &["32x32.png", "128x128.png", "128x128@2x.png", "icon.icns", "icon.ico"] {
        let target_path = icons_dir.join(name);
        if !target_path.exists() {
            fs::copy(&icon_path, &target_path)?;
        }
    }
    
    Ok(())
} 