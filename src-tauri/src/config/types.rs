use serde::{Deserialize, Serialize};

/// Permission level for a shared folder
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    ReadOnly,
    ReadWrite,
}

impl Default for Permission {
    fn default() -> Self {
        Permission::ReadOnly
    }
}

/// A folder shared with AI agents via MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFolder {
    pub path: String,
    pub permission: Permission,
    pub enabled: bool,
    /// Whether the folder path currently exists on disk
    #[serde(default = "default_true")]
    pub available: bool,
}

fn default_true() -> bool {
    true
}

/// Application-wide configuration persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub folders: Vec<SharedFolder>,
    /// Maximum file size in MB that the MCP server will serve (default: 50)
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

/// Result of scanning a folder for supported/unsupported files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderScanResult {
    pub total_files: usize,
    pub supported_files: usize,
    pub unsupported_files: usize,
    pub unsupported_list: Vec<String>,
}

/// File category for type filtering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileCategory {
    Code,
    Text,
    Data,
    Document,
    Image,
    Unsupported,
}
