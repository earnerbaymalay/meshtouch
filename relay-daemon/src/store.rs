//! Message storage and cleanup tasks

use sqlx::SqlitePool;
use std::time::Duration;
use tracing::{info, error};

/// Background task to clean up expired messages
pub async fn start_cleanup_task(pool: SqlitePool, interval_mins: u64) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_mins * 60));
    
    info!("Starting expired message cleanup task (interval: {} mins)", interval_mins);

    loop {
        interval.tick().await;
        
        match crate::db::cleanup_expired(&pool).await {
            Ok(count) => {
                if count > 0 {
                    info!("Cleaned up {} expired messages", count);
                }
            }
            Err(e) => {
                error!("Failed to clean up expired messages: {}", e);
            }
        }
    }
}
