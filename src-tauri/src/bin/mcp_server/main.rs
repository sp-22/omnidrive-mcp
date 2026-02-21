//! OmniDrive Server — Standalone MCP server binary

mod sandbox;
pub mod tools; 
pub mod config;
mod activity;
mod sse;

use rmcp::{ServerHandler, ServiceExt, transport::stdio};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::model::*;
use rmcp::tool_handler;

use config::load_config;
use config::AppConfig;

use std::sync::Arc;
use tokio::sync::RwLock;
use notify::Watcher;

/// The OmniDrive server handler
#[derive(Clone)]
pub struct OmniDriveServer {
    pub config: Arc<RwLock<AppConfig>>,
    pub tool_router: ToolRouter<Self>,
}

#[tool_handler]
impl ServerHandler for OmniDriveServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "OmniDrive provides secure access to user-specified local files.\n\
                 Tools: list_directory, read_file, write_file, search_files, \
                 grep_content, read_lines, move_file, delete_file, copy_file, \
                 get_file_info, batch_read, zip_files, unzip_files, patch_file."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}

/// Detect the name of the parent process (the MCP client)
fn detect_parent_name() -> String {
    use sysinfo::{System, Pid};
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let pid = Pid::from(std::process::id() as usize);
    if let Some(process) = sys.process(pid) {
        if let Some(parent_pid) = process.parent() {
            if let Some(parent) = sys.process(parent_pid) {
                return parent.name().to_string();
            }
        }
    }
    
    // Fallback to env var
    std::env::var("MCP_CLIENT_NAME").unwrap_or_else(|_| "Generic MCP Client".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[OmniDrive] Starting OmniDrive Server v0.1.0...");

    // Try to detect parent process name immediately
    let parent_name = detect_parent_name();
    activity::set_agent_name(parent_name);

    let app_config = load_config();
    eprintln!("[OmniDrive] Loaded config: {} folders", app_config.folders.len());

    let server = OmniDriveServer::new(app_config);
    let server_config = server.config.clone();

    // Set up file watcher for live config reloads
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let config_path = config::get_config_path();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            if event.kind.is_modify() || event.kind.is_create() {
                let _ = tx.blocking_send(());
            }
        }
    })?;

    if let Some(parent) = config_path.parent() {
        watcher.watch(parent, notify::RecursiveMode::NonRecursive)?;
    }

    // Background task to handle reloads
    tokio::spawn(async move {
        while let Some(_) = rx.recv().await {
            // Debounce or small delay to ensure file is written fully
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            
            eprintln!("[OmniDrive] Config file change detected. Reloading...");
            let new_config = load_config();
            let mut config = server_config.write().await;
            *config = new_config;
            eprintln!("[OmniDrive] Config reloaded successfully ({} folders).", config.folders.len());
        }
    });

    let mut use_sse = false;
    let mut port: u16 = 3199;
    let mut allowed_origins = vec![
        "https://chatgpt.com".to_string(),
        "https://gemini.google.com".to_string(),
        "https://claude.ai".to_string(),
        "https://aistudio.google.com".to_string(),
    ];

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--transport" => {
                i += 1;
                if i < args.len() && args[i] == "sse" {
                    use_sse = true;
                }
            }
            "--port" => {
                i += 1;
                if i < args.len() {
                    if let Ok(p) = args[i].parse() {
                        port = p;
                    }
                }
            }
            "--allowed-origins" => {
                i += 1;
                if i < args.len() {
                    let arg = &args[i];
                    if !arg.trim().is_empty() {
                        allowed_origins = arg.split(',').map(|s| s.trim().to_string()).collect();
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    if use_sse {
        activity::log_connect();
        let _watcher = watcher; 
        sse::start_sse_server(server, port, allowed_origins).await?;
    } else {
        eprintln!("[OmniDrive] Server ready. Listening on stdio.");
        
        // Log connection when spinning up
        activity::log_connect();

        let service = server.serve(stdio())
            .await
            .inspect_err(|e| eprintln!("[OmniDrive] Error: {}", e))?;

        // Keep watcher alive
        let _watcher = watcher;

        service.waiting().await?;
    }
    
    Ok(())
}
