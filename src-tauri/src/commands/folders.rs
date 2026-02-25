use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

use crate::config::types::{AppConfig, FolderScanResult, Permission, SharedFolder};
use crate::config::store::write_shared_config;
use crate::file_filter;

/// Application state holding the current config, protected by a Mutex
pub struct AppState {
    pub config: Mutex<AppConfig>,
}

/// Scan a folder and return counts of supported vs unsupported files
fn scan_folder(path: &str) -> Result<FolderScanResult, String> {
    let dir_path = Path::new(path);
    if !dir_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !dir_path.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    let mut total = 0;
    let mut supported = 0;
    let mut unsupported = 0;
    let mut unsupported_list = Vec::new();

    // Walk the directory (non-recursive for performance on large dirs)
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                total += 1;
                let filename = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if file_filter::is_file_supported(filename) {
                    supported += 1;
                } else {
                    unsupported += 1;
                    if unsupported_list.len() < 50 {
                        // Cap the list to avoid huge payloads
                        unsupported_list.push(filename.to_string());
                    }
                }
            }
        }
    }

    Ok(FolderScanResult {
        total_files: total,
        supported_files: supported,
        unsupported_files: unsupported,
        unsupported_list,
    })
}

/// Persist the current config to both tauri-plugin-store and the shared config file
fn persist_config(config: &AppConfig) -> Result<(), String> {
    write_shared_config(config)
}

/// Check if a new folder path overlaps with existing folders (nesting)
fn check_folder_overlap(existing: &[SharedFolder], new_path: &str) -> Option<String> {
    let new_canonical = fs::canonicalize(new_path).ok()?;
    let new_str = new_canonical.to_string_lossy();

    for folder in existing {
        if let Ok(existing_canonical) = fs::canonicalize(&folder.path) {
            let existing_str = existing_canonical.to_string_lossy();

            if new_str.starts_with(existing_str.as_ref()) || existing_str.starts_with(new_str.as_ref()) {
                return Some(folder.path.clone());
            }
        }
    }
    None
}

/// Add a folder to the shared folders list
#[tauri::command]
pub fn add_folder(
    state: State<'_, AppState>,
    path: String,
) -> Result<FolderScanResult, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;

    // Check if folder already exists
    if config.folders.iter().any(|f| f.path == path) {
        return Err("Folder is already shared".to_string());
    }

    // Validate path exists
    if !Path::new(&path).exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    if !Path::new(&path).is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    // Check for folder nesting overlap
    if let Some(overlapping) = check_folder_overlap(&config.folders, &path) {
        return Err(format!(
            "OVERLAP:This folder overlaps with an existing shared folder: {}. Consider using the parent folder instead.",
            overlapping
        ));
    }

    // Scan the folder for supported/unsupported files
    let scan = scan_folder(&path)?;

    // Add the folder
    config.folders.push(SharedFolder {
        path: path.clone(),
        permission: Permission::ReadOnly,
        enabled: true,
        available: true,
    });

    // Persist config
    persist_config(&config)?;

    Ok(scan)
}

/// Remove a folder from the shared folders list
#[tauri::command]
pub fn remove_folder(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;

    let initial_len = config.folders.len();
    config.folders.retain(|f| f.path != path);

    if config.folders.len() == initial_len {
        return Err("Folder not found".to_string());
    }

    persist_config(&config)?;
    Ok(())
}

/// List all shared folders with their current status
#[tauri::command]
pub fn list_folders(state: State<'_, AppState>) -> Result<Vec<SharedFolder>, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;

    // Validate availability of each folder
    for folder in &mut config.folders {
        folder.available = Path::new(&folder.path).exists() && Path::new(&folder.path).is_dir();
    }

    Ok(config.folders.clone())
}

/// Toggle permission for a specific folder
#[tauri::command]
pub fn toggle_permission(
    state: State<'_, AppState>,
    path: String,
    permission: Permission,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;

    if let Some(folder) = config.folders.iter_mut().find(|f| f.path == path) {
        folder.permission = permission;
        persist_config(&config)?;
        Ok(())
    } else {
        Err("Folder not found".to_string())
    }
}

/// Toggle enabled/disabled for a specific folder
#[tauri::command]
pub fn toggle_folder_enabled(
    state: State<'_, AppState>,
    path: String,
    enabled: bool,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;

    if let Some(folder) = config.folders.iter_mut().find(|f| f.path == path) {
        folder.enabled = enabled;
        persist_config(&config)?;
        Ok(())
    } else {
        Err("Folder not found".to_string())
    }
}

/// Scan a folder for file type breakdown
#[tauri::command]
pub fn scan_folder_files(path: String) -> Result<FolderScanResult, String> {
    scan_folder(&path)
}

/// Get the path to the MCP server binary (for connection info)
#[tauri::command]
pub fn get_omnidrive_path() -> Result<String, String> {
    // In development, the binary is in the target directory
    // In production, it's bundled alongside the app
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;

    let server_name = if cfg!(target_os = "windows") {
        "omnidrive_server.exe"
    } else {
        "omnidrive_server"
    };

    // Look for the server binary next to the main executable
    if let Some(parent) = current_exe.parent() {
        let server_path = parent.join(server_name);
        if server_path.exists() {
            return Ok(server_path.to_string_lossy().to_string());
        }
    }

    // Fallback: return the expected name
    Ok(server_name.to_string())
}

/// Get the full app config
#[tauri::command]
pub fn get_app_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

/// Update max file size setting
#[tauri::command]
pub fn update_max_file_size(
    state: State<'_, AppState>,
    max_size_mb: u32,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.max_file_size_mb = max_size_mb;
    persist_config(&config)?;
    Ok(())
}
