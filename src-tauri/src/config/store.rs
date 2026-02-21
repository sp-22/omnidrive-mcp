use std::fs;
use std::path::PathBuf;
use serde_json;
use crate::config::types::AppConfig;

/// Returns the path to the shared config file that the MCP sidecar reads.
/// Located at: ~/.omnidrive/config.json
pub fn get_shared_config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".omnidrive").join("config.json")
}

/// Write the current AppConfig to the shared config file so the MCP sidecar can read it.
pub fn write_shared_config(config: &AppConfig) -> Result<(), String> {
    let path = get_shared_config_path();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// Read the shared config file. Returns a default config if the file doesn't exist.
pub fn read_shared_config() -> AppConfig {
    let path = get_shared_config_path();

    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}
