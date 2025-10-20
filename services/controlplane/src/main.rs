use std::path::Path;

use proto::api::controlplane::control_plane_service_server::ControlPlaneServiceServer;
use tonic::transport::Server;

mod config;
mod server;
mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::ServerConfig::from_env();
    let control_plane = server::ControlPlane::new(Path::new(config::DB_PATH)).await?;

    println!("ControlPlaneService listening on {}", config.addr);
    println!("Database at: {}", config::DB_PATH);

    Server::builder()
        .add_service(ControlPlaneServiceServer::new(control_plane))
        .serve(config.addr)
        .await?;

    Ok(())
}
