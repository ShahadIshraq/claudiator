#![allow(missing_docs)]

mod apns;
mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod router;
mod utils;

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::sync::Mutex;

use clap::Parser;

use config::ServerConfig;
use db::pool;
use router::AppState;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
#[allow(clippy::expect_used)]
async fn main() {
    let config = ServerConfig::parse();

    // Build env filter: RUST_LOG takes precedence, then config.log_level
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    // File appender with daily rotation
    let file_appender = tracing_appender::rolling::daily(&config.log_dir, "server.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_target(true))
        .with(fmt::layer().with_ansi(false).with_writer(non_blocking))
        .init();

    // Initialize database
    let db_pool = pool::create_pool(&config.db_path).expect("Failed to create database pool");

    db::migrations::run(&db_pool).expect("Failed to run database migrations");

    // Load version counters from metadata table
    let (data_version, notification_version) = {
        let conn = db_pool.get().expect("Failed to get db connection");
        let data_v = db::queries::get_metadata(&conn, "data_version")
            .ok()
            .flatten()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let notif_v = db::queries::get_metadata(&conn, "notification_version")
            .ok()
            .flatten()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        (data_v, notif_v)
    };

    tracing::info!(
        "Loaded data_version: {}, notification_version: {}",
        data_version,
        notification_version
    );

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
        master_key: config.api_key.clone(),
        db_pool,
        version: AtomicU64::new(data_version),
        notification_version: AtomicU64::new(notification_version),
        last_cleanup: AtomicU64::new(0),
        apns_client,
        retention_events_days: config.retention_events_days,
        retention_sessions_days: config.retention_sessions_days,
        retention_devices_days: config.retention_devices_days,
        auth_failures: Arc::new(Mutex::new(HashMap::new())),
    });

    let app = router::build_router(state);

    let addr = format!("{}:{}", config.bind, config.port);
    tracing::info!("Claudiator server starting on {}", addr);
    tracing::info!("Database: {}", config.db_path);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    tracing::info!("Server ready, waiting for events...");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("Server error");
}

#[allow(clippy::expect_used)]
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("Shutdown signal received, finishing in-flight requests...");
}
