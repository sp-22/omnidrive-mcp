use std::sync::Mutex;
use std::fs;
use tauri_plugin_shell::{ShellExt, process::CommandChild};
use tauri::AppHandle;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
struct PairingConfig {
    approved_origins: Vec<String>,
}

lazy_static::lazy_static! {
    static ref SSE_PROCESS: Mutex<Option<CommandChild>> = Mutex::new(None);
    static ref SSE_PORT: Mutex<u16> = Mutex::new(0);
}

#[derive(Serialize)]
pub struct SseStatus {
    pub running: bool,
    pub port: u16,
    pub url: Option<String>,
}

#[tauri::command]
pub fn start_sse_mode(app: AppHandle, port: u16, allowed_origins: Vec<String>) -> Result<SseStatus, String> {
    let mut process_guard = SSE_PROCESS.lock().unwrap();
    let mut port_guard = SSE_PORT.lock().unwrap();

    if process_guard.is_some() {
        return Ok(SseStatus {
            running: true,
            port: *port_guard,
            url: Some(format!("http://127.0.0.1:{}/sse", *port_guard)),
        });
    }

    // Build args
    let mut args = vec![
        "--transport".to_string(),
        "sse".to_string(),
        "--port".to_string(),
        port.to_string(),
    ];

    if !allowed_origins.is_empty() {
        args.push("--allowed-origins".to_string());
        args.push(allowed_origins.join(","));
    }

    let shell = app.shell();
    let command = shell.sidecar("omnidrive-server").map_err(|e| e.to_string())?
        .args(args);

    let (mut rx, child) = command.spawn().map_err(|e| e.to_string())?;

    // Drain output in background
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                tauri_plugin_shell::process::CommandEvent::Stdout(line) => {
                    println!("[SSE] {}", String::from_utf8_lossy(&line));
                }
                tauri_plugin_shell::process::CommandEvent::Stderr(line) => {
                    eprintln!("[SSE] {}", String::from_utf8_lossy(&line));
                }
                _ => {}
            }
        }
    });

    *process_guard = Some(child);
    *port_guard = port;

    Ok(SseStatus {
        running: true,
        port,
        url: Some(format!("http://127.0.0.1:{}/sse", port)),
    })
}

#[tauri::command]
pub fn stop_sse_mode() -> Result<SseStatus, String> {
    let mut process_guard = SSE_PROCESS.lock().unwrap();
    if let Some(child) = process_guard.take() {
        let _ = child.kill();
    }
    
    let mut port_guard = SSE_PORT.lock().unwrap();
    *port_guard = 0;

    Ok(SseStatus {
        running: false,
        port: 0,
        url: None,
    })
}

#[tauri::command]
pub fn get_sse_status() -> Result<SseStatus, String> {
    let process_guard = SSE_PROCESS.lock().unwrap();
    let port = *SSE_PORT.lock().unwrap();
    
    if process_guard.is_some() {
        Ok(SseStatus {
            running: true,
            port,
            url: Some(format!("http://127.0.0.1:{}/sse", port)),
        })
    } else {
        Ok(SseStatus {
            running: false,
            port: 0,
            url: None,
        })
    }
}

#[tauri::command]
pub fn approve_origin(origin: String) -> Result<(), String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let path = home.join(".omnidrive").join("pairings.json");
    
    let mut config = if path.exists() {
        let contents = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str::<PairingConfig>(&contents).map_err(|e| e.to_string())?
    } else {
        PairingConfig::default()
    };
    
    if !config.approved_origins.contains(&origin) {
        config.approved_origins.push(origin);
        let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn get_approved_origins() -> Result<Vec<String>, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let path = home.join(".omnidrive").join("pairings.json");
    
    if !path.exists() {
        return Ok(Vec::new());
    }
    
    let contents = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let config = serde_json::from_str::<PairingConfig>(&contents).map_err(|e| e.to_string())?;
    Ok(config.approved_origins)
}

#[tauri::command]
pub fn revoke_origin(origin: String) -> Result<(), String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let path = home.join(".omnidrive").join("pairings.json");
    
    if !path.exists() {
        return Ok(());
    }
    
    let contents = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut config = serde_json::from_str::<PairingConfig>(&contents).map_err(|e| e.to_string())?;
    
    config.approved_origins.retain(|o| o != &origin);
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    
    Ok(())
}

