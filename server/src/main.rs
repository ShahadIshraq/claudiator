mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod router;

use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use clap::Parser;

use config::ServerConfig;
use db::pool;
use router::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = ServerConfig::parse();

    // Initialize database
    let db_pool = pool::create_pool(&config.db_path)
        .expect("Failed to create database pool");

    db::migrations::run(&db_pool).expect("Failed to run database migrations");

    let state = Arc::new(AppState {
        api_key: config.api_key.clone(),
        db_pool,
        version: AtomicU64::new(0),
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
