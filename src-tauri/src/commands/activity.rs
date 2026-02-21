use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub id: String,
    pub timestamp: String,
    pub tool: String,
    pub category: String,
    pub path: Option<String>,
    pub agent: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectedAgent {
    pub name: String,
    pub last_seen: String,
    pub status: String, // "connected" or "disconnected"
}

lazy_static::lazy_static! {
    static ref ACTIVITY_FILE: PathBuf = {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".omnidrive");
        path.push("activity.jsonl");
        path
    };
}

/// Read and parse the activity log file
fn read_all_logs() -> Vec<ActivityEntry> {
    let content = fs::read_to_string(&*ACTIVITY_FILE).unwrap_or_default();
    let mut entries: Vec<ActivityEntry> = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    
    // Sort newest first
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    entries
}

#[tauri::command]
pub async fn get_activity_log(
    limit: usize,
    offset: usize,
    category: Option<String>,
) -> Result<Vec<ActivityEntry>, String> {
    let entries = read_all_logs();
    
    let filtered: Vec<ActivityEntry> = if let Some(cat) = category {
        if cat == "all" || cat.is_empty() {
            entries
        } else {
            entries.into_iter().filter(|e| e.category == cat).collect()
        }
    } else {
        entries
    };

    let paged = filtered
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    Ok(paged)
}

#[tauri::command]
pub async fn get_connected_agents() -> Result<Vec<ConnectedAgent>, String> {
    let entries = read_all_logs();
    let mut agents_map = std::collections::HashMap::new();

    // Find the most recent timestamp for each agent
    for entry in entries {
        agents_map.entry(entry.agent.clone())
            .or_insert_with(|| entry.timestamp.clone());
    }

    let now = Utc::now();
    let timeout = Duration::minutes(5);

    let mut agents: Vec<ConnectedAgent> = agents_map.into_iter().map(|(name, last_seen)| {
        let status = if let Ok(dt) = last_seen.parse::<DateTime<Utc>>() {
            if now.signed_duration_since(dt) < timeout {
                "connected"
            } else {
                "disconnected"
            }
        } else {
            "disconnected"
        };

        ConnectedAgent {
            name,
            last_seen,
            status: status.to_string(),
        }
    }).collect();

    agents.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(agents)
}

#[tauri::command]
pub async fn clear_activity_log() -> Result<(), String> {
    fs::write(&*ACTIVITY_FILE, "").map_err(|e| e.to_string())?;
    Ok(())
}
