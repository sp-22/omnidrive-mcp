import { invoke } from "@tauri-apps/api/core";
import { AppConfig, FolderScanResult, Permission, SharedFolder, ActivityEntry, ConnectedAgent, SseStatus } from "./types";

/// --- Folder Management ---

export async function addFolder(path: string): Promise<FolderScanResult> {
    return invoke<FolderScanResult>("add_folder", { path });
}

export async function removeFolder(path: string): Promise<void> {
    return invoke<void>("remove_folder", { path });
}

export async function listFolders(): Promise<SharedFolder[]> {
    return invoke<SharedFolder[]>("list_folders");
}

export async function togglePermission(path: string, permission: Permission): Promise<void> {
    return invoke<void>("toggle_permission", { path, permission });
}

export async function toggleFolderEnabled(path: string, enabled: boolean): Promise<void> {
    return invoke<void>("toggle_folder_enabled", { path, enabled });
}

export async function scanFolderFiles(path: string): Promise<FolderScanResult> {
    return invoke<FolderScanResult>("scan_folder_files", { path });
}

/// --- Config ---

export async function getMcpServerPath(): Promise<string> {
    return invoke<string>("get_omnidrive_path");
}

export async function getAppConfig(): Promise<AppConfig> {
    return invoke<AppConfig>("get_app_config");
}

export async function updateMaxFileSize(maxSizeMb: number): Promise<void> {
    return invoke<void>("update_max_file_size", { maxSizeMb });
}

/// --- Activity Tracking ---

export async function getActivityLog(limit: number, offset: number, category?: string): Promise<ActivityEntry[]> {
    return invoke<ActivityEntry[]>("get_activity_log", { limit, offset, category });
}

export async function getConnectedAgents(): Promise<ConnectedAgent[]> {
    return invoke<ConnectedAgent[]>("get_connected_agents");
}

export async function clearActivityLog(): Promise<void> {
    return invoke<void>("clear_activity_log");
}

/// --- SSE ---

export async function startSseMode(port: number, allowedOrigins: string[]): Promise<SseStatus> {
    return invoke<SseStatus>("start_sse_mode", { port, allowedOrigins });
}

export async function stopSseMode(): Promise<SseStatus> {
    return invoke<SseStatus>("stop_sse_mode");
}

export async function getSseStatus(): Promise<SseStatus> {
    return invoke<SseStatus>("get_sse_status");
}

export async function approveOrigin(origin: string): Promise<void> {
    return invoke<void>("approve_origin", { origin });
}

export async function revokeOrigin(origin: string): Promise<void> {
    return invoke<void>("revoke_origin", { origin });
}

export async function getApprovedOrigins(): Promise<string[]> {
    return invoke<string[]>("get_approved_origins");
}
