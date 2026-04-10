//! MeshRelay — Encrypted Message Relay Daemon
//!
//! Stores and forwards E2E-encrypted messages between users.
//! The relay NEVER sees plaintext — it only handles ciphertext.
//!
//! API:
//!   POST /api/v1/messages       — Store a message for a recipient
//!   GET  /api/v1/messages/{id}  — Fetch pending messages
//!   POST /api/v1/register       — Register a public key
//!   GET  /api/v1/health         — Health check
//!   POST /api/v1/relay/forward  — Relay-to-relay message forwarding

mod api;
mod config;
mod db;
mod models;
mod store;

use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use config::RelayConfig;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "meshtouch-relay", version, about = "MeshRelay encrypted message relay")]
struct Cli {
    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "meshtouch_relay=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    // Load config
    let config = RelayConfig::load(&cli.config).unwrap_or_else(|e| {
        tracing::error!("Failed to load config: {}", e);
        std::process::exit(1);
    });

    tracing::info!(
        "MeshRelay v{} starting",
        env!("CARGO_PKG_VERSION")
    );
    tracing::info!("Relay ID: {}", config.relay_id);
    tracing::info!("Database: {}", config.database_path);
    tracing::info!("Listen: {}:{}", config.host, config.port);

    // Initialize database
    let pool = db::init_pool(&config.database_path)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        });

    // Run migrations
    db::migrate(&pool)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Failed to run migrations: {}", e);
            std::process::exit(1);
        });

    // Start background tasks
    let cleanup_pool = pool.clone();
    let cleanup_interval = config.cleanup_interval_minutes;
    tokio::spawn(async move {
        store::start_cleanup_task(cleanup_pool, cleanup_interval).await;
    });

    // Build application state
    let state = api::AppState {
        pool,
        config: config.clone(),
    };

    // Build router
    let app = Router::new()
        .route("/api/v1/health", get(api::health_check))
        .route("/api/v1/register", post(api::register_key))
        .route("/api/v1/messages", post(api::store_message))
        .route("/api/v1/messages/{recipient_id}", get(api::fetch_messages))
        .route("/api/v1/messages/{message_id}/ack", post(api::ack_message))
        .route("/api/v1/relay/forward", post(api::relay_forward))
        .route("/api/v1/relay/peers", get(api::list_peers))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid listen address");

    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
