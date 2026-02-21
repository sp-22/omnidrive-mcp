use axum::{Router, middleware::{self, Next}, response::Response, body::Body, http::{Request, StatusCode}};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService,
    session::local::LocalSessionManager,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer, AllowOrigin};
use axum::http::{header::{ACCEPT, CONTENT_TYPE, ORIGIN}, Method};
use std::fs;
use serde::{Deserialize, Serialize};

use crate::OmniDriveServer;
use crate::activity;

#[derive(Serialize, Deserialize, Default)]
struct PairingConfig {
    approved_origins: Vec<String>,
}

fn get_pairings_path() -> std::path::PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    home.join(".omnidrive").join("pairings.json")
}

fn is_origin_approved(origin: &str) -> bool {
    // Basic built-in whitelist for testing or we can just rely on the file
    let path = get_pairings_path();
    if let Ok(contents) = fs::read_to_string(path) {
        if let Ok(config) = serde_json::from_str::<PairingConfig>(&contents) {
            return config.approved_origins.contains(&origin.to_string());
        }
    }
    false
}

async fn pairing_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let origin = req.headers().get(ORIGIN).and_then(|v| v.to_str().ok());
    
    if let Some(origin_str) = origin {
        if !is_origin_approved(origin_str) {
            eprintln!("[OmniDrive] Blocked unauthenticated origin: {}", origin_str);
            activity::log_activity(
                "system",
                "security",
                None,
                &format!("Blocked connection attempt from unapproved origin: {}", origin_str),
            );
            return Err(StatusCode::FORBIDDEN);
        }
    }
    
    Ok(next.run(req).await)
}

pub async fn start_sse_server(
    server: OmniDriveServer,
    port: u16,
    allowed_origins: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure the pairings file exists if it doesn't
    let pairings_path = get_pairings_path();
    if !pairings_path.exists() {
        let _ = fs::create_dir_all(pairings_path.parent().unwrap());
        let _ = fs::write(&pairings_path, r#"{"approved_origins": []}"#);
    }

    let config = StreamableHttpServerConfig {
        stateful_mode: false,
        ..Default::default()
    };
    
    let http_service: StreamableHttpService<McpDriveServer, LocalSessionManager> = 
        StreamableHttpService::new(
            move || Ok(server.clone()),
            Default::default(),
            config
        );
    
    let mut cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE, ACCEPT]);

    if allowed_origins.is_empty() {
        cors = cors.allow_origin(Any);
    } else {
        let origins: Vec<axum::http::HeaderValue> = allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        cors = cors.allow_origin(AllowOrigin::list(origins));
    }

    let app = Router::new()
        .nest_service("/sse", http_service)
        .layer(middleware::from_fn(pairing_middleware))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("[OmniDrive] Starting SSE transport on http://{}/sse", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
