use serde::{Deserialize, Serialize};

/// A registered user's public key info
pub struct User {
    pub recipient_id: String,
    pub public_key: String,
    pub fingerprint: String,
    pub alias: Option<String>,
    pub registered_at: chrono::DateTime<chrono::Utc>,
}

/// A pending message (ciphertext only — relay can't read it)
#[derive(Serialize, Deserialize)]
pub struct PendingMessage {
    pub message_id: String,
    pub sender_id: String,
    pub ciphertext: String,
    pub sender_ratchet_key: String,
    pub msg_num: i64,
    pub stored_at: String,
    pub expires_at: String,
    pub delivered: bool,
}

/// Message receipt for the sender
#[derive(Serialize)]
pub struct MessageReceipt {
    pub message_id: String,
    pub stored_at: String,
    pub expires_at: String,
}
