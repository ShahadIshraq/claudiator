mod apns;
mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod router;

use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use clap::Parser;

use config::ServerConfig;
use db::pool;
use router::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = ServerConfig::parse();

    // Initialize database
    let db_pool = pool::create_pool(&config.db_path).expect("Failed to create database pool");

    db::migrations::run(&db_pool).expect("Failed to run database migrations");

    // Build APNs client if configured
    let apns_client = if let (Some(key_path), Some(key_id), Some(team_id), Some(bundle_id)) = (
        &config.apns_key_path,
        &config.apns_key_id,
        &config.apns_team_id,
        &config.apns_bundle_id,
    ) {
        match apns::ApnsClient::new(
            key_path,
            key_id.clone(),
            team_id.clone(),
            bundle_id.clone(),
            config.apns_sandbox,
        ) {
            Ok(client) => {
                tracing::info!("APNs client initialized (sandbox: {})", config.apns_sandbox);
                Some(Arc::new(client))
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to initialize APNs client: {}. Push notifications disabled.",
                    e
                );
                None
            }
        }
    } else {
        tracing::info!("APNs not configured, push notifications disabled");
        None
    };

    let state = Arc::new(AppState {
        api_key: config.api_key.clone(),
        db_pool,
        version: AtomicU64::new(0),
        notification_version: AtomicU64::new(0),
        apns_client,
    });

    let app = router::build_router(state);

    let addr = format!("{}:{}", config.bind, config.port);
    tracing::info!("Claudiator server starting on {}", addr);
    tracing::info!("Database: {}", config.db_path);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    tracing::info!("Server ready, waiting for events...");

    axum::serve(listener, app).await.expect("Server error");
}
