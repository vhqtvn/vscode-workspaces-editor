document.addEventListener('DOMContentLoaded', () => {
    // Get DOM elements
    const profilePathInput = document.getElementById('profile-path');
    const loadBtn = document.getElementById('load-btn');
    const refreshBtn = document.getElementById('refresh-btn');
    const addBtn = document.getElementById('add-btn');
    const workspacesList = document.getElementById('workspaces-list');
    const workspaceDetailsContent = document.getElementById('workspace-details-content');
    
    // Set default VSCode profile path (platform-specific)
    if (navigator.appVersion.indexOf('Win') !== -1) {
        // Windows default
        profilePathInput.value = '%APPDATA%\\Code';
    } else if (navigator.appVersion.indexOf('Mac') !== -1) {
        // macOS default
        profilePathInput.value = '~/Library/Application Support/Code';
    } else {
        // Linux default
        profilePathInput.value = '~/.config/Code';
    }
    
    // Load default profile path from backend
    window.__TAURI__.invoke('get_default_profile_path')
        .then(result => {
            if (typeof result === 'string') {
                profilePathInput.value = result;
            }
        })
        .catch(err => console.error('Could not get default profile path:', err));
    
    // Load workspaces
    loadBtn.addEventListener('click', async () => {
        const profilePath = profilePathInput.value.trim();
        if (!profilePath) {
            alert('Please enter a VSCode profile path');
            return;
        }
        
        try {
            // Call Rust backend to load workspaces
            // Using the Tauri API when it's available
            workspacesList.innerHTML = '<div class="loading">Loading workspaces...</div>';
            
            const result = await window.__TAURI__.invoke('get_workspaces', { profile_path: profilePath });
            
            // Check if the result is an array (success) or a string (error)
            if (Array.isArray(result)) {
                displayWorkspaces(result);
            } else {
                console.error('Error from backend:', result);
                alert(`Failed to load workspaces: ${result}`);
                workspacesList.innerHTML = '<div class="error-message">Error loading workspaces. See console for details.</div>';
            }
        } catch (error) {
            console.error('Error loading workspaces:', error);
            alert(`Failed to load workspaces: ${error}`);
            workspacesList.innerHTML = '<div class="error-message">Error loading workspaces. See console for details.</div>';
        }
    });
    
    // Refresh workspaces
    refreshBtn.addEventListener('click', () => {
        loadBtn.click();
    });
    
    // Add workspace
    addBtn.addEventListener('click', async () => {
        const workspacePath = prompt('Enter workspace path:');
        if (!workspacePath) return;
        
        try {
            const result = await window.__TAURI__.invoke('add_workspace', { 
                profile_path: profilePathInput.value.trim(),
                workspace_path: workspacePath 
            });
            
            if (result === true) {
                alert('Workspace added successfully');
                loadBtn.click(); // Refresh the list
            } else {
                alert(`Failed to add workspace: ${result}`);
            }
        } catch (error) {
            console.error('Error adding workspace:', error);
            alert(`Failed to add workspace: ${error}`);
        }
    });
    
    // Display workspaces in the list
    function displayWorkspaces(workspaces) {
        workspacesList.innerHTML = '';
        
        if (!workspaces || workspaces.length === 0) {
            workspacesList.innerHTML = '<div class="empty-message">No workspaces found.</div>';
            return;
        }
        
        workspaces.forEach(workspace => {
            const workspaceItem = document.createElement('div');
            workspaceItem.className = 'workspace-item';
            workspaceItem.textContent = workspace.name || workspace.path;
            workspaceItem.dataset.id = workspace.id;
            
            workspaceItem.addEventListener('click', () => {
                // Remove active class from all items
                document.querySelectorAll('.workspace-item').forEach(item => {
                    item.classList.remove('active');
                });
                
                // Add active class to clicked item
                workspaceItem.classList.add('active');
                
                // Display workspace details
                displayWorkspaceDetails(workspace);
            });
            
            workspacesList.appendChild(workspaceItem);
        });
    }
    
    // Display workspace details
    function displayWorkspaceDetails(workspace) {
        workspaceDetailsContent.innerHTML = `
            <div class="detail-item">
                <strong>ID:</strong> ${workspace.id}
            </div>
            <div class="detail-item">
                <strong>Name:</strong> ${workspace.name || 'N/A'}
            </div>
            <div class="detail-item">
                <strong>Path:</strong> ${workspace.path}
            </div>
            <div class="detail-item">
                <strong>Last Used:</strong> ${new Date(workspace.last_used * 1000).toLocaleString()}
            </div>
            <div class="actions">
                <button id="edit-btn">Edit</button>
                <button id="delete-btn">Delete</button>
                <button id="open-btn">Open in VSCode</button>
            </div>
        `;
        
        // Add event listeners for action buttons
        const editBtn = workspaceDetailsContent.querySelector('#edit-btn');
        const deleteBtn = workspaceDetailsContent.querySelector('#delete-btn');
        const openBtn = workspaceDetailsContent.querySelector('#open-btn');
        
        editBtn.addEventListener('click', () => editWorkspace(workspace));
        deleteBtn.addEventListener('click', () => deleteWorkspace(workspace));
        openBtn.addEventListener('click', () => openWorkspace(workspace));
    }
    
    // Edit workspace
    async function editWorkspace(workspace) {
        const newName = prompt('Enter new workspace name:', workspace.name);
        if (newName === null) return; // User cancelled
        
        try {
            const result = await window.__TAURI__.invoke('edit_workspace', {
                profile_path: profilePathInput.value.trim(),
                workspace_id: workspace.id,
                new_name: newName
            });
            
            if (result === true) {
                alert('Workspace updated successfully');
                loadBtn.click(); // Refresh the list
            } else {
                alert(`Failed to update workspace: ${result}`);
            }
        } catch (error) {
            console.error('Error updating workspace:', error);
            alert(`Failed to update workspace: ${error}`);
        }
    }
    
    // Delete workspace
    async function deleteWorkspace(workspace) {
        if (!confirm(`Are you sure you want to delete workspace "${workspace.name || workspace.path}"?`)) {
            return;
        }
        
        try {
            const result = await window.__TAURI__.invoke('delete_workspace', {
                profile_path: profilePathInput.value.trim(),
                workspace_id: workspace.id
            });
            
            if (result === true) {
                alert('Workspace deleted successfully');
                loadBtn.click(); // Refresh the list
                workspaceDetailsContent.innerHTML = '<div class="empty-message">Select a workspace to view details.</div>';
            } else {
                alert(`Failed to delete workspace: ${result}`);
            }
        } catch (error) {
            console.error('Error deleting workspace:', error);
            alert(`Failed to delete workspace: ${error}`);
        }
    }
    
    // Open workspace in VSCode
    async function openWorkspace(workspace) {
        try {
            const result = await window.__TAURI__.invoke('open_workspace', {
                workspace_path: workspace.path,
                original_path: workspace.original_path
            });
            
            if (result !== true) {
                alert(`Failed to open workspace: ${result}`);
            }
        } catch (error) {
            console.error('Error opening workspace:', error);
            alert(`Failed to open workspace: ${error}`);
        }
    }
}); 