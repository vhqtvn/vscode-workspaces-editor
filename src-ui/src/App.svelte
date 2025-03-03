<script lang="ts">
  import { stopPropagation } from 'svelte/legacy';

  import { onMount } from 'svelte';
  import type { Workspace, FilterOptions, ThreeState } from './types.js';
  import { workspace_exists, get_workspace_type, is_remote_workspace, get_workspace_label, get_workspace_tags } from './types.js';
  import { invoke } from '@tauri-apps/api/core';

  let profilePath = $state('');
  let workspaces: Workspace[] = $state([]);
  let selectedWorkspace: Workspace | null = $state(null);
  let loading = $state(false);
  let error: string | null = $state(null);
  let initialized = $state(false);
  let searchQuery = $state('');
  let markedForDeletion: Set<string> = $state(new Set());
  let workspaceExistsMap: Map<string, boolean> = $state(new Map());
  let filterOptions = $state({
    existing: 'any' as ThreeState,
    remote: 'any' as ThreeState,
    type: 'all',
    tag: ''
  });
  let hoveredWorkspace: Workspace | null = $state(null);
  let lastViewedWorkspace: Workspace | null = $state(null);
  let showDeleteConfirmation = $state(false);
  let selectedIndex = $state(0);
  let workspacesToDelete: Workspace[] = $state([]);
  let knownPaths: string[] = $state([]);
  let isCustomPath = $state(false);

  async function checkWorkspaceExists(workspace: Workspace) {
    try {
      const exists = await workspace_exists(workspace);
      workspaceExistsMap.set(workspace.id, exists);
      // Force reactivity update
      workspaceExistsMap = new Map(workspaceExistsMap);
    } catch (err) {
      console.error('Failed to check workspace existence:', err);
    }
  }

  async function loadWorkspaces() {
    if (!initialized || !profilePath) {
      console.log('Not initialized or missing profile path');
      return;
    }
    
    console.log('Loading workspaces with profile path:', profilePath);
    loading = true;
    error = null;
    
    try {
      console.log('Invoking get_workspaces command...');
      const result = await invoke<Workspace[]>('get_workspaces', { profilePath });
      console.log('Received workspaces:', result);
      workspaces = result;
      
      // Clear and rebuild the exists map
      workspaceExistsMap.clear();
      // Check existence for all workspaces
      await Promise.all(workspaces.map(checkWorkspaceExists));
    } catch (err) {
      console.error('Failed to load workspaces:', err);
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  async function refreshWorkspaces() {
    if (profilePath) {
      await loadWorkspaces();
    }
  }

  async function openWorkspace(workspace: Workspace) {
    if (!initialized) return;
    
    console.log('Opening workspace:', workspace);
    try {
      await invoke('open_workspace', { 
        workspace_path: workspace.path,
        original_path: workspace.original_path
      });
    } catch (err) {
      console.error('Failed to open workspace:', err);
      error = err instanceof Error ? err.message : String(err);
    }
  }

  async function deleteWorkspace(workspace: Workspace) {
    if (!initialized) return;
    
    console.log('Deleting workspace:', workspace);
    try {
      await invoke('delete_workspace', { 
        profile_path: profilePath,
        workspace_id: workspace.id
      });
      await loadWorkspaces();
    } catch (err) {
      console.error('Failed to delete workspace:', err);
      error = err instanceof Error ? err.message : String(err);
    }
  }

  function getFilteredWorkspaces(): Workspace[] {
    return workspaces.filter(workspace => {
      // Text search
      if (searchQuery) {
        const searchLower = searchQuery.toLowerCase();
        const nameMatch = get_workspace_label(workspace).toLowerCase().includes(searchLower);
        const pathMatch = workspace.path.toLowerCase().includes(searchLower);
        if (!nameMatch && !pathMatch) return false;
      }

      // Filter options
      if (filterOptions.existing !== 'any') {
        const exists = workspaceExistsMap.get(workspace.id) ?? false;
        if (filterOptions.existing === 'yes' && !exists) return false;
        if (filterOptions.existing === 'no' && exists) return false;
      }
      
      if (filterOptions.remote !== 'any') {
        const isRemote = is_remote_workspace(workspace);
        if (filterOptions.remote === 'yes' && !isRemote) return false;
        if (filterOptions.remote === 'no' && isRemote) return false;
      }
      
      if (filterOptions.type !== 'all' && get_workspace_type(workspace) !== filterOptions.type) return false;
      if (filterOptions.tag && !get_workspace_tags(workspace).includes(filterOptions.tag)) return false;

      return true;
    });
  }

  function toggleMarkForDeletion(workspace: Workspace, event?: MouseEvent) {
    if (event) event.stopPropagation();
    
    if (markedForDeletion.has(workspace.id)) {
      markedForDeletion.delete(workspace.id);
    } else {
      markedForDeletion.add(workspace.id);
    }
    // Force reactivity update
    markedForDeletion = new Set(markedForDeletion);
  }

  function showDeleteConfirmationDialog() {
    if (!initialized || markedForDeletion.size === 0) return;
    workspacesToDelete = workspaces.filter(w => markedForDeletion.has(w.id));
    selectedIndex = 0;
    showDeleteConfirmation = true;
  }

  function hideDeleteConfirmation() {
    showDeleteConfirmation = false;
    selectedIndex = 0;
    workspacesToDelete = [];
  }

  async function confirmDelete() {
    if (!initialized || workspacesToDelete.length === 0) return;
    
    loading = true;
    error = null;
    
    try {
      for (const workspace of workspacesToDelete) {
        await invoke('delete_workspace', { 
          profile_path: profilePath,
          workspace_id: workspace.id
        });
      }
      markedForDeletion.clear();
      hideDeleteConfirmation();
      await loadWorkspaces();
    } catch (err) {
      console.error('Failed to delete workspaces:', err);
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  function handleDeleteKeydown(event: KeyboardEvent) {
    if (!showDeleteConfirmation) return;
    
    switch (event.key) {
      case 'ArrowUp':
        selectedIndex = Math.max(0, selectedIndex - 1);
        event.preventDefault();
        break;
      case 'ArrowDown':
        selectedIndex = Math.min(workspacesToDelete.length - 1, selectedIndex + 1);
        event.preventDefault();
        break;
      case 'Enter':
        if (selectedIndex === workspacesToDelete.length) {
          confirmDelete();
        } else if (selectedIndex === workspacesToDelete.length + 1) {
          hideDeleteConfirmation();
        }
        event.preventDefault();
        break;
      case 'Escape':
        hideDeleteConfirmation();
        event.preventDefault();
        break;
    }
  }

  function selectAllWorkspaces() {
    const filtered = getFilteredWorkspaces();
    filtered.forEach(w => markedForDeletion.add(w.id));
    // Force reactivity update
    markedForDeletion = new Set(markedForDeletion);
  }

  function deselectAllWorkspaces() {
    markedForDeletion.clear();
    // Force reactivity update
    markedForDeletion = new Set();
  }

  function onWorkspaceHover(workspace: Workspace | null) {
    hoveredWorkspace = workspace;
    if (workspace) {
      lastViewedWorkspace = workspace;
    }
  }

  async function loadKnownPaths() {
    try {
      const paths = await invoke<string[]>('get_known_vscode_paths');
      knownPaths = paths;
      if (paths.length > 0 && !profilePath) {
        profilePath = paths[0];
      }
    } catch (err) {
      console.error('Failed to load known VSCode paths:', err);
      error = err instanceof Error ? err.message : String(err);
    }
  }

  function handleProfilePathChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    if (target.value === 'custom') {
      isCustomPath = true;
      profilePath = '';
    } else {
      isCustomPath = false;
      profilePath = target.value;
      loadWorkspaces();
    }
  }

  function handleCustomPathInput(event: Event) {
    const target = event.target as HTMLInputElement;
    profilePath = target.value;
  }

  onMount(() => {
    async function initialize() {
      console.log('Component mounted');
      try {
        await loadKnownPaths();
        initialized = true;
        if (profilePath) {
          await loadWorkspaces();
        }
      } catch (err) {
        console.error('Failed to initialize:', err);
        error = err instanceof Error ? err.message : String(err);
      }
    }

    window.addEventListener('keydown', handleDeleteKeydown);
    initialize();

    return () => {
      window.removeEventListener('keydown', handleDeleteKeydown);
    };
  });
</script>

<main>
  <header>
    <h1>VSCode Workspaces Editor</h1>
    <div class="actions">
      <button on:click={refreshWorkspaces}>
        <i class="fas fa-sync"></i> Refresh
      </button>
      <button on:click={selectAllWorkspaces}>
        <i class="fas fa-check-square"></i> Select All
      </button>
      <button on:click={deselectAllWorkspaces}>
        <i class="fas fa-square"></i> Deselect All
      </button>
      <button 
        on:click={showDeleteConfirmationDialog}
        disabled={markedForDeletion.size === 0}
        class="danger"
      >
        <i class="fas fa-trash"></i> Delete Selected ({markedForDeletion.size})
      </button>
    </div>
  </header>

  <div class="toolbar">
    <div class="profile-selector">
      <label for="profile-path">VSCode Profile:</label>
      {#if knownPaths.length > 0}
        <select 
          id="profile-path-select" 
          value={isCustomPath ? 'custom' : profilePath} 
          on:change={handleProfilePathChange}
        >
          {#each knownPaths as path}
            <option value={path}>{path}</option>
          {/each}
          <option value="custom">Custom Path...</option>
        </select>
      {/if}
      {#if isCustomPath}
        <input 
          type="text" 
          id="profile-path" 
          value={profilePath}
          on:input={handleCustomPathInput}
          placeholder="Enter VSCode profile path"
        />
      {/if}
      <button on:click={loadWorkspaces} disabled={loading || !initialized || !profilePath}>
        {loading ? 'Loading...' : 'Load'}
      </button>
    </div>

    <div class="search-filters">
      <div class="search-box">
        <i class="fas fa-search"></i>
        <input 
          type="text" 
          bind:value={searchQuery} 
          placeholder="Search workspaces..."
        />
      </div>

      <div class="filter-options">
        <div class="filter-group">
          <label>Existing:</label>
          <select bind:value={filterOptions.existing}>
            <option value="any">Any</option>
            <option value="yes">Exists</option>
            <option value="no">Missing</option>
          </select>
        </div>

        <div class="filter-group">
          <label>Remote:</label>
          <select bind:value={filterOptions.remote}>
            <option value="any">Any</option>
            <option value="yes">Remote</option>
            <option value="no">Local</option>
          </select>
        </div>

        <div class="filter-group">
          <label>Type:</label>
          <select bind:value={filterOptions.type}>
            <option value="all">All Types</option>
            <option value="folder">Folders</option>
            <option value="file">Files</option>
            <option value="workspace">Workspaces</option>
          </select>
        </div>

        <div class="filter-group">
          <label>Tag:</label>
          <input 
            type="text" 
            bind:value={filterOptions.tag} 
            placeholder="Filter by tag..."
          />
        </div>
      </div>
    </div>
  </div>

  {#if error}
    <div class="error-message">
      <i class="fas fa-exclamation-circle"></i>
      {error}
    </div>
  {/if}

  <div class="content-container">
    <div class="workspaces-container">
      <h2>Workspaces</h2>
      <div class="list">
        {#if !initialized}
          <div class="loading-message">
            <i class="fas fa-spinner fa-spin"></i> Initializing...
          </div>
        {:else if loading}
          <div class="loading-message">
            <i class="fas fa-spinner fa-spin"></i> Loading workspaces...
          </div>
        {:else if workspaces.length === 0}
          <div class="empty-message">
            <i class="fas fa-folder-open"></i>
            No workspaces found. Set the profile path and click Load.
          </div>
        {:else}
          {#each getFilteredWorkspaces() as workspace (workspace.id)}
            {@const typedWorkspace = workspace as Workspace}
            <div 
              class="workspace-item" 
              class:marked={markedForDeletion.has(typedWorkspace.id)}
              on:click={(e: MouseEvent) => {
                toggleMarkForDeletion(typedWorkspace);
              }}
              on:dblclick={() => openWorkspace(typedWorkspace)}
              on:mouseenter={() => onWorkspaceHover(typedWorkspace)}
              on:mouseleave={() => onWorkspaceHover(null)}
            >
              <div class="workspace-checkbox">
                <input 
                  type="checkbox"
                  checked={markedForDeletion.has(typedWorkspace.id)}
                  on:click={(e) => toggleMarkForDeletion(typedWorkspace, e)}
                />
              </div>
              <div class="workspace-icon">
                {#if is_remote_workspace(typedWorkspace)}
                  <i class="fas fa-globe"></i>
                {:else if get_workspace_type(typedWorkspace) === 'folder'}
                  <i class="fas fa-folder"></i>
                {:else if get_workspace_type(typedWorkspace) === 'file'}
                  <i class="fas fa-file"></i>
                {:else}
                  <i class="fas fa-cube"></i>
                {/if}
              </div>
              <div class="workspace-info">
                <div class="workspace-name">
                  {get_workspace_label(typedWorkspace)}
                  {#if !(workspaceExistsMap.get(typedWorkspace.id) ?? false)}
                    <span class="badge danger">Missing</span>
                  {/if}
                  {#if is_remote_workspace(typedWorkspace)}
                    <span class="badge info">
                      {#if typedWorkspace.parsed_info?.remote_user}
                        {typedWorkspace.parsed_info.remote_user}@
                      {/if}
                      {typedWorkspace.parsed_info?.remote_host}
                      {#if typedWorkspace.parsed_info?.remote_port}
                        :{typedWorkspace.parsed_info.remote_port}
                      {/if}
                    </span>
                  {/if}
                  {#if typedWorkspace.parsed_info?.container_path}
                    <span class="badge info">Container</span>
                  {/if}
                  {#each get_workspace_tags(typedWorkspace) as tag}
                    <span class="badge">{tag}</span>
                  {/each}
                </div>
                <div class="workspace-path">{typedWorkspace.path}</div>
              </div>
              <div class="workspace-actions">
                <button 
                  class="icon-button"
                  on:click={stopPropagation(() => openWorkspace(typedWorkspace))}
                  title="Open Workspace"
                >
                  <i class="fas fa-external-link-alt"></i>
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>

    <div class="workspace-details">
      <h2>Workspace Details</h2>
      {#if hoveredWorkspace || lastViewedWorkspace}
        {@const displayWorkspace = (hoveredWorkspace || lastViewedWorkspace) as Workspace}
        <div class="details-container">
          <div class="details-content">
            <div class="detail-row">
              <label>Name:</label>
              <div>{get_workspace_label(displayWorkspace)}</div>
            </div>
            <div class="detail-row">
              <label>Path:</label>
              <div>{displayWorkspace.path}</div>
            </div>
            <div class="detail-row">
              <label>Type:</label>
              <div>{get_workspace_type(displayWorkspace)}</div>
            </div>
            <div class="detail-row">
              <label>Status:</label>
              <div>
                {#if workspaceExistsMap.get(displayWorkspace.id) ?? false}
                  <span class="badge">Exists</span>
                {:else}
                  <span class="badge danger">Missing</span>
                {/if}
                {#if is_remote_workspace(displayWorkspace)}
                  <span class="badge info">Remote</span>
                {/if}
              </div>
            </div>
            {#if is_remote_workspace(displayWorkspace) && displayWorkspace.parsed_info}
              <div class="detail-row">
                <label>Remote Host:</label>
                <div>
                  {#if displayWorkspace.parsed_info.remote_user}
                    {displayWorkspace.parsed_info.remote_user}@
                  {/if}
                  {displayWorkspace.parsed_info.remote_host}
                  {#if displayWorkspace.parsed_info.remote_port}
                    :{displayWorkspace.parsed_info.remote_port}
                  {/if}
                </div>
              </div>
              {#if displayWorkspace.parsed_info.container_path}
                <div class="detail-row">
                  <label>Container Path:</label>
                  <div>{displayWorkspace.parsed_info.container_path}</div>
                </div>
              {/if}
            {/if}
            {#if get_workspace_tags(displayWorkspace).length > 0}
              <div class="detail-row">
                <label>Tags:</label>
                <div class="tags-list">
                  {#each get_workspace_tags(displayWorkspace) as tag}
                    <span class="tag">{tag}</span>
                  {/each}
                </div>
              </div>
            {/if}
            <div class="detail-row">
              <label>Sources:</label>
              <div class="sources-list">
                {#each displayWorkspace.sources as source}
                  <div class="source-item">
                    <span class="source-path">{source}</span>
                  </div>
                {/each}
              </div>
            </div>
          </div>
        </div>
      {:else}
        <div class="empty-message">
          <i class="fas fa-info-circle"></i>
          Hover over a workspace to view details.
        </div>
      {/if}
    </div>
  </div>
</main>

{#if showDeleteConfirmation}
  <div class="modal-overlay">
    <div class="modal delete-confirmation" role="dialog" aria-modal="true">
      <div class="modal-header">
        <h3>Delete Workspaces</h3>
        <p class="modal-subtitle">The following workspaces will be deleted:</p>
      </div>
      <div class="modal-content">
        <div class="workspace-list">
          {#each workspacesToDelete as workspace, i}
            <div 
              class="workspace-item-confirm" 
              class:selected={i === selectedIndex}
              on:click={() => selectedIndex = i}
            >
              <div class="workspace-icon">
                {#if is_remote_workspace(workspace)}
                  <i class="fas fa-globe"></i>
                {:else if get_workspace_type(workspace) === 'folder'}
                  <i class="fas fa-folder"></i>
                {:else if get_workspace_type(workspace) === 'file'}
                  <i class="fas fa-file"></i>
                {:else}
                  <i class="fas fa-cube"></i>
                {/if}
              </div>
              <div class="workspace-info">
                <div class="workspace-name">{get_workspace_label(workspace)}</div>
                <div class="workspace-path">{workspace.path}</div>
              </div>
            </div>
          {/each}
        </div>
        <div class="modal-actions">
          <button 
            class="danger" 
            on:click={confirmDelete}
            class:selected={selectedIndex === workspacesToDelete.length}
          >
            Delete
          </button>
          <button 
            on:click={hideDeleteConfirmation}
            class:selected={selectedIndex === workspacesToDelete.length + 1}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  main {
    padding: 0.75rem;
    height: 100vh;
    max-height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
    padding: 0.5rem;
    background: var(--container-bg-color);
    border-radius: 4px;
    flex-shrink: 0;
  }

  h1 {
    margin: 0;
    font-size: 1.25rem;
    color: #0078d4;
  }

  h2 {
    margin: 0;
    padding: 0.5rem;
    font-size: 1rem;
    color: #444;
    border-bottom: 1px solid #eee;
    flex-shrink: 0;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  .profile-selector {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    margin-bottom: 0.75rem;
    padding: 0.5rem;
    background: var(--container-bg-color);
    border-radius: 4px;
    flex-shrink: 0;
  }

  input {
    flex: 1;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 0.9rem;
  }

  button {
    padding: 0.5rem 1rem;
    background: #0078d4;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: background-color 0.2s;
  }

  button:disabled {
    background: #ccc;
    cursor: not-allowed;
  }

  button:not(:disabled):hover {
    background: #106ebe;
  }

  .error-message {
    padding: 0.5rem;
    margin-bottom: 0.75rem;
    background: #fde7e9;
    color: #c53030;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .content-container {
    display: flex;
    gap: 0.75rem;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .workspaces-container {
    position: relative;
    flex: 1;
    display: flex;
    flex-direction: column;
    background: white;
    border: 1px solid #eee;
    border-radius: 4px;
    overflow: visible;
    min-width: 0;
  }

  .workspace-details {
    width: 350px;
    background: white;
    border: 1px solid #eee;
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    word-break: break-all;
  }

  .list {
    position: relative;
    flex: 1;
    overflow: auto;
    padding: 0.5rem;
  }

  .workspace-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    border-bottom: 1px solid #eee;
    cursor: pointer;
  }

  .workspace-item:hover {
    background: #f5f5f5;
  }

  .workspace-item.selected {
    background: #e3f2fd;
  }

  .workspace-item.marked {
    background: #fff3e0;
  }

  .workspace-checkbox {
    display: flex;
    align-items: center;
    padding: 0 0.25rem;
  }

  .workspace-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    color: #666;
  }

  .workspace-info {
    flex: 1;
    min-width: 0;
  }

  .workspace-name {
    font-weight: 500;
    margin-bottom: 0.25rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .workspace-path {
    font-size: 0.875rem;
    color: #666;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
    font-size: 0.75rem;
    font-weight: 500;
    background: #e0e0e0;
    color: #424242;
  }

  .badge.danger {
    background: #ffebee;
    color: #c62828;
  }

  .badge.info {
    background: #e3f2fd;
    color: #1565c0;
  }

  .icon-button {
    padding: 0.5rem;
    min-width: 2.5rem;
    height: 2.5rem;
    border-radius: 4px;
    border: 1px solid #ddd;
    background: white;
    color: #666;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .icon-button i {
    font-size: 1rem;
  }

  .icon-button:hover {
    background: #f5f5f5;
    border-color: #ccc;
    color: #333;
  }

  .icon-button.danger:hover {
    background: #ffebee;
    border-color: #ef5350;
    color: #c62828;
  }

  .workspace-actions {
    display: flex;
    gap: 0.5rem;
    margin-left: auto;
    padding-left: 0.5rem;
  }

  .tags-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
  }

  .tag {
    display: inline-flex;
    align-items: center;
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
    font-size: 0.75rem;
    background: #e3f2fd;
    color: #1565c0;
  }

  button.danger {
    background: #c62828;
    color: white;
  }

  button.danger:hover {
    background: #b71c1c;
  }

  button.danger:disabled {
    background: #ffcdd2;
    cursor: not-allowed;
  }

  .empty-message {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 2rem;
    color: #666;
    text-align: center;
  }

  .loading-message {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 2rem;
    color: #666;
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    margin-bottom: 0.75rem;
    background: #ffebee;
    color: #c62828;
    border-radius: 4px;
  }

  .toolbar {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 0.75rem;
    padding: 0.75rem;
    background: var(--container-bg-color);
    border-radius: 4px;
    flex-shrink: 0;
  }

  .search-filters {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .search-box {
    position: relative;
    flex: 1;
  }

  .search-box i {
    position: absolute;
    left: 0.5rem;
    top: 50%;
    transform: translateY(-50%);
    color: #666;
  }

  .search-box input {
    width: 100%;
    padding: 0.5rem 0.5rem 0.5rem 2rem;
  }

  .filter-options {
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  .filter-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .filter-group label {
    font-size: 0.875rem;
    color: #666;
    min-width: 4rem;
  }

  .filter-group select {
    padding: 0.375rem 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 0.875rem;
    background: white;
  }

  .filter-group input {
    padding: 0.375rem 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 0.875rem;
    min-width: 150px;
  }

  .sources-list {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .source-item {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    padding: 0.25rem;
    background: #f5f5f5;
    border-radius: 3px;
  }

  .source-type {
    font-weight: 500;
    color: #1565c0;
    padding: 0.125rem 0.375rem;
    background: #e3f2fd;
    border-radius: 3px;
    font-size: 0.75rem;
  }

  .source-path {
    font-size: 0.875rem;
    color: #666;
    word-break: break-all;
  }

  .workspace-sources {
    display: flex;
    gap: 0.25rem;
    margin-top: 0.25rem;
    flex-wrap: wrap;
  }

  .source-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.125rem 0.375rem;
    background: #f0f4f8;
    border-radius: 3px;
    font-size: 0.75rem;
    color: #666;
  }

  .source-badge i {
    font-size: 0.75rem;
    color: #1565c0;
  }

  .details-content {
    padding: 0.75rem;
  }

  .detail-row {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .detail-row:last-child {
    margin-bottom: 0;
  }

  .detail-row label {
    font-weight: 500;
    color: #666;
    min-width: 6rem;
  }

  .sources-list {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    max-height: 200px;
    overflow-y: auto;
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--container-bg-color);
    border-radius: 4px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    width: 600px;
    max-width: 90vw;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    padding: 1rem;
    border-bottom: 1px solid #eee;
  }

  .modal-header h3 {
    margin: 0;
    color: #c62828;
    font-size: 1.25rem;
  }

  .modal-subtitle {
    margin: 0.5rem 0 0;
    color: #666;
    font-size: 0.9rem;
  }

  .modal-content {
    padding: 1rem;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .workspace-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-height: 400px;
    overflow-y: auto;
    padding: 0.5rem;
    background: #f5f5f5;
    border-radius: 4px;
  }

  .workspace-item-confirm {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: white;
    border-radius: 4px;
    border: 1px solid #eee;
    cursor: pointer;
  }

  .workspace-item-confirm.selected {
    background: #ffebee;
    border-color: #ef5350;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid #eee;
  }

  .modal-actions button {
    min-width: 100px;
  }

  .modal-actions button.selected {
    outline: 2px solid #0078d4;
    outline-offset: 2px;
  }

  .modal-actions button.danger.selected {
    outline-color: #c62828;
  }
</style> 