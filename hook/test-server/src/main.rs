use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use clap::Parser;
use colored::Colorize;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "test-server", about = "Claudiator test server")]
struct Args {
    #[arg(long, default_value = "3000")]
    port: u16,
    #[arg(long, default_value = "test-key")]
    api_key: String,
}

struct AppState {
    api_key: String,
}

fn check_auth(
    headers: &axum::http::HeaderMap,
    state: &AppState,
) -> Result<(), axum::http::StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let expected = format!("Bearer {}", state.api_key);
    if auth_header == expected {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn ping_handler(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    if let Err(status) = check_auth(&headers, &state) {
        return (
            status,
            Json(serde_json::json!({
                "error": "unauthorized",
                "message": "Invalid or missing API key"
            })),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "server_version": "test-0.1.0"
        })),
    )
        .into_response()
}

async fn events_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Err(status) = check_auth(&headers, &state) {
        return (
            status,
            Json(serde_json::json!({
                "error": "unauthorized",
                "message": "Invalid or missing API key"
            })),
        )
            .into_response();
    }

    // Extract and log event information
    let timestamp = Utc::now().to_rfc3339();
    println!(
        "{}",
        format!("[{}] EVENT received", timestamp).green().bold()
    );

    // Device info
    let device_name = body["device"]["device_name"].as_str().unwrap_or("unknown");
    let platform = body["device"]["platform"].as_str().unwrap_or("unknown");
    let device_id = body["device"]["device_id"].as_str().unwrap_or("unknown");
    let device_id_short = if device_id.len() > 8 {
        format!("{}...", &device_id[..8])
    } else {
        device_id.to_string()
    };

    println!(
        "  {}: {} ({}) [{}]",
        "Device".cyan(),
        device_name,
        platform,
        device_id_short
    );

    // Session and event info
    let session_id = body["event"]["session_id"].as_str().unwrap_or("unknown");
    println!("  {}: {}", "Session".cyan(), session_id);

    let hook_event_name = body["event"]["hook_event_name"]
        .as_str()
        .unwrap_or("unknown");
    let notification_type = body["event"]["notification_type"].as_str();
    let event_display = if let Some(notif_type) = notification_type {
        format!("{} ({})", hook_event_name, notif_type)
    } else {
        hook_event_name.to_string()
    };
    println!("  {}: {}", "Event".cyan(), event_display);

    // Optional fields
    if let Some(cwd) = body["event"]["cwd"].as_str() {
        println!("  {}: {}", "CWD".cyan(), cwd);
    }

    if let Some(message) = body["event"]["message"].as_str() {
        println!("  {}: {}", "Message".cyan(), message);
    }

    println!("{}", "---".dimmed());

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok"
        })),
    )
        .into_response()
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let state = Arc::new(AppState {
        api_key: args.api_key.clone(),
    });

    let app = axum::Router::new()
        .route("/api/v1/ping", axum::routing::get(ping_handler))
        .route("/api/v1/events", axum::routing::post(events_handler))
        .with_state(state);

    println!(
        "Claudiator test server running on http://0.0.0.0:{}",
        args.port
    );
    println!("API key: {}", args.api_key);
    println!("Waiting for events...\n");

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port))
        .await
        .expect("Failed to bind port");
    axum::serve(listener, app).await.expect("Server error");
}
