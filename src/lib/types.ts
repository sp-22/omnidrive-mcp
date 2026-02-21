/// TypeScript types mirroring the Rust backend types

export type Permission = "readonly" | "readwrite";
export type ThemePreference = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";
export type AppTab = "folders" | "connectedApps" | "activityLog" | "settings";
export type ServerUiState = "active" | "dormant";

export interface SharedFolder {
    path: string;
    permission: Permission;
    enabled: boolean;
    available: boolean;
}

export interface AppConfig {
    folders: SharedFolder[];
    max_file_size_mb: number;
}

export interface FolderScanResult {
    total_files: number;
    supported_files: number;
    unsupported_files: number;
    unsupported_list: string[];
}

export type ServerStatus = "live" | "offline" | "checking";

export interface ConnectionClient {
    name: string;
    icon: string;
    configPath: string;
    configSnippet: string;
}

export interface ConnectedAgent {
    name: string;
    last_seen: string;
    status: "connected" | "disconnected";
}

export type ActivityCategory = "all" | "read" | "write" | "delete" | "security" | "system" | "connect";

export interface ActivityEntry {
    id: string;
    timestamp: string;
    tool: string;
    category: ActivityCategory;
    path: string | null;
    agent: string;
    summary: string;
}

export interface SseStatus {
    running: boolean;
    port: number;
    url: string | null;
}
