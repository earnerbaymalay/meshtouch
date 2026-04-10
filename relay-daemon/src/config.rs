use serde::Deserialize;
use std::time::Instant;

#[derive(Deserialize)]
pub struct RelayConfig {
    pub relay_id: String,
    pub host: String,
    pub port: u16,
    pub database_path: String,
    pub max_message_size: usize,
    pub max_messages_per_user: usize,
    pub cleanup_interval_minutes: u64,
    pub known_peers: Vec<PeerConfig>,
    #[serde(skip, default = "Instant::now")]
    pub start_time: Instant,
}

impl Clone for RelayConfig {
    fn clone(&self) -> Self {
        Self {
            relay_id: self.relay_id.clone(),
            host: self.host.clone(),
            port: self.port,
            database_path: self.database_path.clone(),
            max_message_size: self.max_message_size,
            max_messages_per_user: self.max_messages_per_user,
            cleanup_interval_minutes: self.cleanup_interval_minutes,
            known_peers: self.known_peers.clone(),
            start_time: Instant::now(), // reset on clone
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct PeerConfig {
    pub id: String,
    pub url: String,
    pub public_key: String,
}

impl RelayConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let mut config: RelayConfig = toml::from_str(&contents)?;
        config.start_time = Instant::now();
        Ok(config)
    }

    pub fn is_known_peer(&self, relay_id: &str) -> bool {
        self.known_peers.iter().any(|p| p.id == relay_id)
    }
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            relay_id: format!("relay_{}", uuid::Uuid::new_v4().simple()),
            host: "0.0.0.0".into(),
            port: 8080,
            database_path: "messages.db".into(),
            max_message_size: 65536, // 64KB max ciphertext
            max_messages_per_user: 1000,
            cleanup_interval_minutes: 60,
            known_peers: vec![],
            start_time: Instant::now(),
        }
    }
}
