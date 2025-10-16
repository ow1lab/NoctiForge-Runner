mod server;
mod config;
mod worker;
mod client;
mod path;

use std::sync::Arc;

use proto::api::worker::worker_service_server::WorkerServiceServer;
use tonic::transport::Server;

use crate::client::controlplane_client::ControlPlaneClient;
use crate::client::registry_clint::RegistryClient;
use crate::server::WorkerServer;
use crate::worker::native::NativeWorker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::ServerConfig::from_env();

    let registry_clinet = RegistryClient::new(config.registry_clinet);
    let controlplane_client = ControlPlaneClient::new(config.controlplane_clinet);
    let function_worker = Arc::new(NativeWorker::new()?);

    let worker_server = WorkerServer::new(
        function_worker,
        controlplane_client,
        registry_clinet 
    );

    println!("ControlPlaneService listening on {}", config.addr);

    Server::builder()
        .add_service(WorkerServiceServer::new(worker_server))
        .serve(config.addr)
        .await?;

    Ok(())
}
