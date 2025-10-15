use anyhow::{Ok, Result};
use proto::api::registry::{registry_service_client::RegistryServiceClient, RegistryPullRequest};
use tonic::Request;

pub struct RegistryClient {
    pub addr: String 
}

impl RegistryClient {
    pub fn new(addr: String) -> Self {
        Self { addr: addr }
    }
}

impl RegistryClient {
    pub async fn get_tar_by_digest(&self, digest: String) -> Result<String> {
        println!("geting tar with digest {}", digest);
        let mut client = RegistryServiceClient::connect(self.addr.clone()).await?;
        let mut response = client.pull(Request::new(RegistryPullRequest { digest } )).await?.into_inner();
        if let Some(next_message) = response.message().await? {
            println!("{:?}", next_message.data);
        }

        Ok("Value".to_string())
    }
}

