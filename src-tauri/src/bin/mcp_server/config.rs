//! Shared config types and reader for the MCP sidecar binary.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFolder {
    pub path: String,
    pub permission: Permission,
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub available: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub folders: Vec<SharedFolder>,
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u32,
}

fn default_max_file_size() -> u32 {
    50
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            folders: Vec::new(),
            max_file_size_mb: 50,
        }
    }
}

/// Get the shared config file path
pub fn get_config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".omnidrive").join("config.json")
}

/// Load config from the shared config file
pub fn load_config() -> AppConfig {
    let path = get_config_path();
    match fs::read_to_string(&path) {
        Ok(contents) => {
            serde_json::from_str(&contents).unwrap_or_else(|e| {
                eprintln!(
                    "[OmniDrive] Failed to parse config at {:?}: {}",
                    path, e
                );
                AppConfig::default()
            })
        }
        Err(_) => {
            eprintln!(
                "[OmniDrive] No config found at {:?}, using defaults",
                path
            );
            AppConfig::default()
        }
    }
}
