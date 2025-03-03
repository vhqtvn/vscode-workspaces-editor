/// <reference types="svelte" />

interface Workspace {
  id: string;
  name: string;
  path: string;
  original_path: string;
  last_used: number;
  storage_path?: string;
  sources: Array<{
    type: 'Storage' | 'Database';
    path: string;
  }>;
  parsed_info?: {
    original_path: string;
    type: string;
    remote_authority?: string;
    remote_host?: string;
    remote_user?: string;
    remote_port?: number;
    path: string;
    container_path?: string;
    label?: string;
    tags: string[];
  };
} 