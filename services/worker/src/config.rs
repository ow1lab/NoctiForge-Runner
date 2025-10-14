use std::net::SocketAddr;

pub struct ServerConfig {
    pub addr: SocketAddr,
    pub controlplane_clinet: String
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let addr = std::env::var("SERVER_ADDR")
            .unwrap_or_else(|_| "[::1]:50003".to_string())
            .parse()
            .expect("Invalid server address");

        let controlplane_clinet = std::env::var("CONTROLPLANE_CLINET")
            .unwrap_or_else(|_| "http://localhost:50002".to_string())
            .parse()
            .expect("Invalid controlplane address");
        Self { addr, controlplane_clinet }
    }
}

