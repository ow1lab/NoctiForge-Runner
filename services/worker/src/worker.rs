use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Ok, Result};
use tokio::{process::Command, sync::Mutex, time::sleep};
use tonic::Request;
use url::Url;

use crate::client::registry_clint::RegistryClient;
use proto::api::action::{function_runner_service_client::FunctionRunnerServiceClient, InvokeRequest};

const SERVER_STARTUP_TIMEOUT_MS: u64 = 3000;
const SERVER_STARTUP_RETRY_INTERVAL_MS: u64 = 10;

pub struct NativeWorker {
    function_urls: Arc<Mutex<HashMap<String, Url>>>,
    registry_service: RegistryClient
}

impl NativeWorker {
    pub fn new(registry_service: RegistryClient) -> Result<Self> {
        Ok(Self {
            function_urls: Arc::new(Mutex::new(HashMap::new())),
            registry_service
        })
    }
}

impl NativeWorker {
    pub async fn execute(&self, digest: String, body: String) -> Result<String> {
        let uri = self.get_available_handler_uri(digest).await?;
        let mut client = FunctionRunnerServiceClient::connect(uri.to_string()).await?;
        let resp = client
            .invoke(Request::new(InvokeRequest {
                payload: Some(body),
            }))
            .await?
            .into_inner();
        Ok(resp.output)
    }

    
    async fn get_available_handler_uri(&self, digest: String) -> Result<Url> {
        // Check if URL is already cached
        {
            let urls = self.function_urls.lock().await;
            if let Some(url) = urls.get(&digest) {
                return Ok(url.clone());
            }
        }

        let dir_path = self.registry_service.get_tar_by_digest(&digest).await?;
        let url = self.start_handler(dir_path).await?;

        // Insert into cache
        self.function_urls.lock().await.insert(digest, url.clone());
        Ok(url)}

    async fn start_handler(&self, bin_path: PathBuf) -> Result<Url> {
        let bootstrap_path = bin_path.join("bootstrap");

        Command::new(&bootstrap_path)
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn handler: {}", e))?;

        let url = Url::parse("http://localhost:54036")
            .map_err(|e| anyhow::anyhow!("Failed to parse URL: {}", e))?;

        self.wait_for_server_ready(&url).await?;
        Ok(url)
    }

    async fn wait_for_server_ready(&self, url: &Url) -> Result<()> {
        let max_attempts = (SERVER_STARTUP_TIMEOUT_MS / SERVER_STARTUP_RETRY_INTERVAL_MS) as u32;
        let retry_interval = Duration::from_millis(SERVER_STARTUP_RETRY_INTERVAL_MS);  

        for attempt in 1..= max_attempts {
            match FunctionRunnerServiceClient::connect(url.to_string()).await {
                std::result::Result::Ok(_) => {
                    println!("Server is ready after {} attempts", attempt);
                    return Ok(());
                }
                Err(e) if attempt == max_attempts => {
                    return Err(anyhow::anyhow!(
                        "Server failed to start after {} attempts: {}",
                        max_attempts,
                        e
                    ));
                }
                Err(_) => {
                    sleep(retry_interval).await;
                }
            }
        }
        
        unreachable!("Loop should have returned")
    }
}
