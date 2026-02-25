use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use chrono::Utc;

/// Core log entry written to the JSONL file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    /// A unique ID for the entry
    pub id: String,
    /// ISO 8601 timestamp string
    pub timestamp: String,
    /// Which tool was used? e.g., "read_file"
    pub tool: String,
    /// General category: "read", "write", "delete"
    pub category: String,
    /// Primary path acted upon (if any)
    pub path: Option<String>,
    /// Who did it? e.g., "Claude Desktop"
    pub agent: String,
    /// Human-readable summary
    pub summary: String,
}

lazy_static::lazy_static! {
    static ref ACTIVITY_DIR: PathBuf = {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".omnidrive");
        path
    };

    static ref ACTIVITY_FILE: PathBuf = ACTIVITY_DIR.join("activity.jsonl");

    static ref CURRENT_AGENT: Mutex<String> = Mutex::new("Generic MCP Client".to_string());

    static ref LOG_MUTEX: Mutex<()> = Mutex::new(());
}

/// Max size of log file before we truncate (e.g., 2 MB)
const MAX_LOG_SIZE_BYTES: u64 = 2 * 1024 * 1024;

pub fn set_agent_name(name: String) {
    if let Ok(mut agent) = CURRENT_AGENT.lock() {
        *agent = name;
    }
}

pub fn get_agent_name() -> String {
    CURRENT_AGENT.lock().unwrap().clone()
}

/// Log an action to the central JSONL file
pub fn log_activity(
    tool: &str,
    category: &str,
    path: Option<&str>,
    summary: &str,
) {
    // Fire and forget, don't crash main server if logging fails
    let _ = try_log_activity(tool, category, path, summary);
}

fn try_log_activity(
    tool: &str,
    category: &str,
    path: Option<&str>,
    summary: &str,
) -> Result<(), std::io::Error> {
    let _guard = LOG_MUTEX.lock().unwrap();

    if !ACTIVITY_DIR.exists() {
        fs::create_dir_all(&*ACTIVITY_DIR)?;
    }

    // Check size for rotation
    if let Ok(meta) = fs::metadata(&*ACTIVITY_FILE) {
        if meta.len() > MAX_LOG_SIZE_BYTES {
            rotate_log()?;
        }
    }

    let entry = ActivityEntry {
        id: uuid::Uuid::new_v4().to_string(), // Requires uuid crate
        timestamp: Utc::now().to_rfc3339(),
        tool: tool.to_string(),
        category: category.to_string(),
        path: path.map(|s| s.to_string()),
        agent: get_agent_name(),
        summary: summary.to_string(),
    };

    let json = serde_json::to_string(&entry)?;
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&*ACTIVITY_FILE)?;
        
    writeln!(file, "{}", json)?;
    Ok(())
}

fn rotate_log() -> Result<(), std::io::Error> {
    // Simple rotation: keep the last 500 lines
    let content = fs::read_to_string(&*ACTIVITY_FILE).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();
    
    let keep_count = 500;
    if lines.len() > keep_count {
        let kept_lines = &lines[lines.len() - keep_count..];
        let new_content = kept_lines.join("\n") + "\n";
        fs::write(&*ACTIVITY_FILE, new_content)?;
    }
    Ok(())
}

pub fn log_connect() {
    log_activity(
        "system",
        "system",
        None,
        &format!("{} linked via MCP", get_agent_name()),
    );
}
