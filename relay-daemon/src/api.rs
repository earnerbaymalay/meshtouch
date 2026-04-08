use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db;
use crate::models::*;

/// Shared application state
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub config: crate::config::RelayConfig,
}

// ── Health Check ──────────────────────────────────────────────

pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let message_count = db::count_all_messages(&state.pool).await.unwrap_or(0);
    let user_count = db::count_all_users(&state.pool).await.unwrap_or(0);

    Json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        relay_id: state.config.relay_id.clone(),
        messages_stored: message_count,
        users_registered: user_count,
        uptime_seconds: state.config.start_time.elapsed().as_secs(),
    })
}

// ── Register Public Key ──────────────────────────────────────

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub public_key: String,    // Hex-encoded public key
    pub fingerprint: String,   // Key fingerprint for display
    pub alias: Option<String>, // Human-readable name
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub recipient_id: String,
    pub fingerprint: String,
}

pub async fn register_key(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, (StatusCode, String)> {
    // Validate key format
    if req.public_key.len() < 32 {
        return Err((StatusCode::BAD_REQUEST, "Invalid public key".into()));
    }

    let recipient_id = Uuid::new_v4().to_string();

    db::register_user(
        &state.pool,
        &recipient_id,
        &req.public_key,
        &req.fingerprint,
        req.alias.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!(
        "User registered: {} (fingerprint: {})",
        recipient_id,
        req.fingerprint
    );

    Ok(Json(RegisterResponse {
        recipient_id: recipient_id.clone(),
        fingerprint: req.fingerprint,
    }))
}

// ── Store Message ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct StoreMessageRequest {
    pub recipient_id: String,     // Who this message is for
    pub sender_id: String,        // Who sent it
    pub ciphertext: String,       // Base64-encoded encrypted payload
    pub sender_ratchet_key: String, // Sender's current ratchet key (for decrypt)
    pub msg_num: i64,             // Message number in the ratchet chain
    pub ttl_hours: Option<i64>,   // Time to live (default: 72h)
}

#[derive(Serialize)]
pub struct StoreMessageResponse {
    pub message_id: String,
    pub stored_at: String,
    pub expires_at: String,
}

pub async fn store_message(
    State(state): State<AppState>,
    Json(req): Json<StoreMessageRequest>,
) -> Result<Json<StoreMessageResponse>, (StatusCode, String)> {
    // Validate recipient exists
    let recipient = db::get_user(&state.pool, &req.recipient_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if recipient.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Recipient not found: {}", req.recipient_id),
        ));
    }

    let message_id = Uuid::new_v4().to_string();
    let ttl = req.ttl_hours.unwrap_or(72);

    db::store_message(
        &state.pool,
        &message_id,
        &req.recipient_id,
        &req.sender_id,
        &req.ciphertext,
        &req.sender_ratchet_key,
        req.msg_num,
        ttl,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!(
        "Message stored: {} → {} ({} bytes ciphertext)",
        req.sender_id,
        req.recipient_id,
        req.ciphertext.len()
    );

    let stored_at = chrono::Utc::now();
    let expires_at = stored_at + chrono::Duration::hours(ttl);

    Ok(Json(StoreMessageResponse {
        message_id: message_id.clone(),
        stored_at: stored_at.to_rfc3339(),
        expires_at: expires_at.to_rfc3339(),
    }))
}

// ── Fetch Messages ────────────────────────────────────────────

#[derive(Serialize)]
pub struct FetchMessagesResponse {
    pub recipient_id: String,
    pub messages: Vec<PendingMessage>,
    pub count: usize,
}

pub async fn fetch_messages(
    State(state): State<AppState>,
    Path(recipient_id): Path<String>,
) -> Result<Json<FetchMessagesResponse>, (StatusCode, String)> {
    let messages = db::get_pending_messages(&state.pool, &recipient_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let count = messages.len();

    Ok(Json(FetchMessagesResponse {
        recipient_id: recipient_id.clone(),
        messages,
        count,
    }))
}

// ── Acknowledge Message (mark as delivered) ───────────────────

pub async fn ack_message(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
) -> Result<StatusCode, String> {
    db::acknowledge_message(&state.pool, &message_id)
        .await
        .map_err(|e| e.to_string())?;

    tracing::debug!("Message acknowledged: {}", message_id);
    Ok(StatusCode::NO_CONTENT)
}

// ── Relay-to-Relay Forwarding ─────────────────────────────────

#[derive(Deserialize)]
pub struct RelayForwardRequest {
    pub message_id: String,
    pub recipient_id: String,
    pub sender_id: String,
    pub ciphertext: String,
    pub sender_ratchet_key: String,
    pub msg_num: i64,
    pub ttl_hours: i64,
    pub source_relay_id: String,
}

pub async fn relay_forward(
    State(state): State<AppState>,
    Json(req): Json<RelayForwardRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Verify source relay is a known peer
    if !state.config.is_known_peer(&req.source_relay_id) {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Unknown relay peer".into(),
        ));
    }

    // Store the forwarded message
    db::store_message(
        &state.pool,
        &req.message_id,
        &req.recipient_id,
        &req.sender_id,
        &req.ciphertext,
        &req.sender_ratchet_key,
        req.msg_num,
        req.ttl_hours,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tracing::info!(
        "Relay forward: {} from {} for {}",
        req.message_id,
        req.source_relay_id,
        req.recipient_id
    );

    Ok(StatusCode::CREATED)
}

// ── List Peer Relays ──────────────────────────────────────────

#[derive(Serialize)]
pub struct PeerInfo {
    pub relay_id: String,
    pub url: String,
    pub last_seen: Option<String>,
    pub status: String,
}

pub async fn list_peers(State(state): State<AppState>) -> Json<Vec<PeerInfo>> {
    let peers = state
        .config
        .known_peers
        .iter()
        .map(|p| PeerInfo {
            relay_id: p.id.clone(),
            url: p.url.clone(),
            last_seen: None, // TODO: Track last successful contact
            status: "unknown".into(),
        })
        .collect();

    Json(peers)
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub relay_id: String,
    pub messages_stored: i64,
    pub users_registered: i64,
    pub uptime_seconds: u64,
}
