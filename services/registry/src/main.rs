use tonic::transport::Server;
use proto::api::registry::registry_service_server::RegistryServiceServer;

mod path;
mod registry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50001".parse().unwrap();
    let file_engine = registry::LocalBackend::default();

    println!("RegistryServiceServer listening on {addr}");

    Server::builder()
        .add_service(RegistryServiceServer::new(file_engine))
        .serve(addr)
        .await?;

    Ok(())
}
