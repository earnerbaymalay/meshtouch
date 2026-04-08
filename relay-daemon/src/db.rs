use sqlx::{sqlite::SqlitePool, Row};

use crate::models::*;

/// Initialize SQLite connection pool
pub async fn init_pool(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let url = format!("sqlite:{}", db_path);
    SqlitePool::connect(&url).await
}

/// Run database migrations
pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            recipient_id TEXT PRIMARY KEY,
            public_key TEXT NOT NULL,
            fingerprint TEXT NOT NULL,
            alias TEXT,
            registered_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            message_id TEXT PRIMARY KEY,
            recipient_id TEXT NOT NULL,
            sender_id TEXT NOT NULL,
            ciphertext TEXT NOT NULL,
            sender_ratchet_key TEXT NOT NULL,
            msg_num INTEGER NOT NULL,
            stored_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            expires_at DATETIME NOT NULL,
            delivered INTEGER DEFAULT 0,
            FOREIGN KEY (recipient_id) REFERENCES users(recipient_id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_messages_recipient
        ON messages(recipient_id, delivered);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_messages_expires
        ON messages(expires_at);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Register a new user's public key
pub async fn register_user(
    pool: &SqlitePool,
    recipient_id: &str,
    public_key: &str,
    fingerprint: &str,
    alias: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO users (recipient_id, public_key, fingerprint, alias)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(recipient_id)
    .bind(public_key)
    .bind(fingerprint)
    .bind(alias)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get user info by recipient_id
pub async fn get_user(pool: &SqlitePool, recipient_id: &str) -> Result<Option<User>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT recipient_id, public_key, fingerprint, alias, registered_at
        FROM users WHERE recipient_id = ?
        "#,
    )
    .bind(recipient_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| User {
        recipient_id: r.get("recipient_id"),
        public_key: r.get("public_key"),
        fingerprint: r.get("fingerprint"),
        alias: r.get("alias"),
        registered_at: r.get("registered_at"),
    }))
}

/// Store an encrypted message
pub async fn store_message(
    pool: &SqlitePool,
    message_id: &str,
    recipient_id: &str,
    sender_id: &str,
    ciphertext: &str,
    sender_ratchet_key: &str,
    msg_num: i64,
    ttl_hours: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO messages (message_id, recipient_id, sender_id, ciphertext,
                            sender_ratchet_key, msg_num, expires_at)
        VALUES (?, ?, ?, ?, ?, ?, datetime('now', ? || ' hours'))
        "#,
    )
    .bind(message_id)
    .bind(recipient_id)
    .bind(sender_id)
    .bind(ciphertext)
    .bind(sender_ratchet_key)
    .bind(msg_num)
    .bind(ttl_hours)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get all pending (undelivered) messages for a recipient
pub async fn get_pending_messages(
    pool: &SqlitePool,
    recipient_id: &str,
) -> Result<Vec<PendingMessage>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT message_id, sender_id, ciphertext, sender_ratchet_key,
               msg_num, stored_at, expires_at, delivered
        FROM messages
        WHERE recipient_id = ? AND delivered = 0
          AND expires_at > datetime('now')
        ORDER BY stored_at ASC
        "#,
    )
    .bind(recipient_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| PendingMessage {
            message_id: r.get("message_id"),
            sender_id: r.get("sender_id"),
            ciphertext: r.get("ciphertext"),
            sender_ratchet_key: r.get("sender_ratchet_key"),
            msg_num: r.get("msg_num"),
            stored_at: r.get("stored_at"),
            expires_at: r.get("expires_at"),
            delivered: r.get::<bool, _>("delivered"),
        })
        .collect())
}

/// Mark a message as delivered
pub async fn acknowledge_message(pool: &SqlitePool, message_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE messages SET delivered = 1 WHERE message_id = ?
        "#,
    )
    .bind(message_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete expired messages (cleanup job)
pub async fn cleanup_expired(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM messages WHERE expires_at <= datetime('now')
        "#,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Count all messages
pub async fn count_all_messages(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM messages")
        .fetch_one(pool)
        .await?;
    Ok(row.get("count"))
}

/// Count all registered users
pub async fn count_all_users(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM users")
        .fetch_one(pool)
        .await?;
    Ok(row.get("count"))
}
