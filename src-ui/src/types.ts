import { invoke } from '@tauri-apps/api/core';

export type ThreeState = 'any' | 'yes' | 'no';

export interface Workspace {
    id: string;
    name?: string;
    path: string;
    original_path: string;
    last_used: number;
    storage_path?: string;
    sources: WorkspaceSource[];
    parsed_info?: WorkspacePathInfo;
}

export interface WorkspacePathInfo {
    original_path: string;
    workspace_type: WorkspaceType;
    remote_authority?: string;
    remote_host?: string;
    remote_user?: string;
    remote_port?: number;
    path: string;
    container_path?: string;
    label?: string;
    tags: string[];
}

export type WorkspaceType = 'folder' | 'file' | 'workspace';

export interface WorkspaceSource {
    type: string;
    path: string;
}

export interface FilterOptions {
    existing: ThreeState;
    remote: ThreeState;
    type: 'all' | 'folder' | 'file' | 'workspace';
    tag: string;
}

// Helper functions
export async function workspace_exists(workspace: Workspace): Promise<boolean> {
    try {
        return await invoke<boolean>('workspace_exists', { workspace });
    } catch (err) {
        console.error('Failed to check if workspace exists:', err);
        // For remote workspaces, we assume they exist
        if (workspace.parsed_info?.remote_authority) {
            return true;
        }
        // For local paths, assume they don't exist if we can't check
        return false;
    }
}

export function get_workspace_type(workspace: Workspace): WorkspaceType {
    return workspace.parsed_info?.workspace_type || 'folder';
}

export function is_remote_workspace(workspace: Workspace): boolean {
    return Boolean(workspace.parsed_info?.remote_authority);
}

export function get_workspace_label(workspace: Workspace): string {
    if (workspace.name && workspace.name.length > 0) {
        return workspace.name;
    }
    if (workspace.parsed_info?.label) {
        return workspace.parsed_info.label;
    }
    // Extract basename from path
    return workspace.path.split(/[\/\\]/).pop() || workspace.path;
}

export function get_workspace_tags(workspace: Workspace): string[] {
    return workspace.parsed_info?.tags || [];
} 