# Plan: Implement "zed-default" Fake Profile

This plan outlines the steps to implement a "zed-default" profile that exclusively lists Zed workspaces, separating them from the standard VSCode workspace list.

## 1. Modify `src/workspaces/paths.rs`

We need to ensure "zed-default" appears in the list of known profiles, even though it's not a real directory.

*   **File**: `src/workspaces/paths.rs`
*   **Function**: `get_known_vscode_paths`
*   **Action**:
    *   Locate the filtering logic where paths are checked for existence:
        ```rust
        paths = paths
            .into_iter()
            .filter(|p| std::path::Path::new(p).is_dir())
            .collect::<Vec<_>>();
        ```
    *   Insert "zed-default" into the `paths` vector *after* this filter step so it isn't removed.
    *   Example:
        ```rust
        // ... existing filtering ...
        
        // Add fake profiles
        paths.push("zed-default".to_string());

        debug!("Found {} known VSCode paths", paths.len());
        paths
        ```

## 2. Modify `src/workspaces/mod.rs`

We need to handle the "zed-default" profile specifically in the workspace retrieval logic and remove the automatic merging of Zed workspaces into standard profiles.

*   **File**: `src/workspaces/mod.rs`
*   **Function**: `get_workspaces`
*   **Action**:
    *   Add a check at the beginning of the function:
        ```rust
        if profile_path == "zed-default" {
            info!("Getting workspaces from Zed default profile");
            return crate::workspaces::zed::get_zed_workspaces();
        }
        ```
    *   **Remove** the existing block that merges Zed workspaces. Look for and delete:
        ```rust
        // Get workspaces from Zed
        match crate::workspaces::zed::get_zed_workspaces() {
            Ok(mut zed_workspaces) => {
                info!("Found {} Zed workspaces", zed_workspaces.len());
                workspaces.append(&mut zed_workspaces);
            }
            Err(e) => {
                warn!("Failed to get Zed workspaces: {}", e);
            }
        }
        ```

## 3. Verification

*   Run `cargo run -- list --profile zed-default` to verify it lists Zed workspaces.
*   Run `cargo run -- list` (default profile) to verify it *no longer* lists Zed workspaces.
*   Run the TUI (`cargo run`) and check if "zed-default" appears in the profile selection list (if implemented in UI) or can be passed via `--profile`.