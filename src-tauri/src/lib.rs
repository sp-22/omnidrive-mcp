mod config;
mod commands;
mod file_filter;

use commands::AppState;
use config::store::read_shared_config;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load existing config or create default
    let initial_config = read_shared_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            config: Mutex::new(initial_config),
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_folder,
            commands::remove_folder,
            commands::list_folders,
            commands::toggle_permission,
            commands::toggle_folder_enabled,
            commands::scan_folder_files,
            commands::get_omnidrive_path,
            commands::get_app_config,
            commands::update_max_file_size,
            commands::activity::get_activity_log,
            commands::activity::get_connected_agents,
            commands::activity::clear_activity_log,
            commands::sse::start_sse_mode,
            commands::sse::stop_sse_mode,
            commands::sse::get_sse_status,
            commands::sse::approve_origin,
            commands::sse::revoke_origin,
            commands::sse::get_approved_origins,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
