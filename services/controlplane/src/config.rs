use std::net::SocketAddr;

// TODO: share this location with registry
pub const DB_PATH: &str = "/var/lib/noctiforge/controlplane/digests.db";

pub struct ServerConfig {
    pub addr: SocketAddr,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let addr = std::env::var("SERVER_ADDR")
            .unwrap_or_else(|_| "[::1]:50002".to_string())
            .parse()
            .expect("Invalid server address");
        Self { addr }
    }
}
