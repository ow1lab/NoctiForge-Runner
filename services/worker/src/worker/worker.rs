use std::{
    collections::HashMap, path::PathBuf, sync::Arc, time::Duration
};

use anyhow::{Ok, Result};
use libcontainer::syscall::Syscall;
use tokio::{sync::Mutex, time::sleep};
use tonic::Request;
use tracing::{debug, info, instrument, warn};
use url::Url;

use crate::{client::registry_clint::RegistryClient, worker::{container, spec::SysUserParms}};
use proto::api::action::{
    InvokeRequest, function_runner_service_client::FunctionRunnerServiceClient,
};

const SERVER_STARTUP_TIMEOUT_MS: u64 = 3000;
const SERVER_STARTUP_RETRY_INTERVAL_MS: u64 = 10;

pub struct Config {
    pub is_dev: bool,
}

pub struct NativeWorker {
    function_urls: Arc<Mutex<HashMap<String, Url>>>,
    registry_service: RegistryClient,
    root_path: PathBuf,
    sysuser: SysUserParms 
}

impl NativeWorker {
    pub fn new(
        registry_service: RegistryClient,
        root_path: PathBuf,
        syscall: &dyn Syscall,
        server_config: Config) -> Result<Self> {
        info!(is_dev = server_config.is_dev, "Creating NativeWorker");
        Ok(Self {
            function_urls: Arc::new(Mutex::new(HashMap::new())),
            registry_service,
            root_path,
            sysuser: SysUserParms {
                uid: syscall.get_euid().as_raw(),
                gid: syscall.get_egid().as_raw(),
            },
        })
    }
}

impl NativeWorker {
    #[instrument(name = "function_execute", level = "debug", skip(self, body), fields(digest = %digest, body_size = body.len()))]
    pub async fn execute(&self, digest: String, body: String) -> Result<String> {
        debug!("Executing function");

        let socket_path = self.get_available_handler_uri(digest.clone()).await?;
        let uri = format!("unix://{}", socket_path.path());

        debug!(uri = %uri, "Connecting to function handler");
        let mut client = FunctionRunnerServiceClient::connect(uri).await.map_err(|e| {
            warn!(digest = %digest, error = %e, "Failed to connect to function handler");
            e
        })?;

        let resp = client
            .invoke(Request::new(InvokeRequest {
                payload: Some(body),
            }))
            .await
            .map_err(|e| {
                warn!(digest = %digest, error = %e, "Function invocation failed");
                e
            })?
            .into_inner();

        debug!(digest = %digest, output_size = resp.output.len(), "Function execution completed");
        Ok(resp.output)
    }

    async fn get_available_handler_uri(&self, digest: String) -> Result<Url> {
        // Check if URL is already cached
        {
            let urls = self.function_urls.lock().await;
            if let Some(url) = urls.get(&digest) {
                debug!(digest = %digest, socket = %url.path(), "Using cached handler");
                return Ok(url.clone());
            }
        }

        info!(digest = %digest, "Starting new handler");
        let dir_path = self.registry_service.get_tar_by_digest(&digest).await?;

        let mut proc = container::ProccesContainer::new(
            self.root_path.clone(),
            dir_path,
            &self.sysuser).await?;
        proc.start()?;

        let url = proc.get_url();
        self.wait_for_server_ready(&url).await?;

        // Insert into cache
        self.function_urls.lock().await.insert(digest.clone(), url.clone());
        info!(digest = %digest, socket = %url.path(), "Handler cached");

        Ok(url)
    }

    async fn wait_for_server_ready(&self, url: &Url) -> Result<()> {
        let max_attempts = (SERVER_STARTUP_TIMEOUT_MS / SERVER_STARTUP_RETRY_INTERVAL_MS) as u32;
        let retry_interval = Duration::from_millis(SERVER_STARTUP_RETRY_INTERVAL_MS);

        debug!(max_attempts, "Waiting for server to be ready");

        for attempt in 1..=max_attempts {
            let uri = format!("unix://{}", url.path());
            match FunctionRunnerServiceClient::connect(uri).await {
                std::result::Result::Ok(_) => {
                    debug!(attempts = attempt, "Server is ready");
                    return Ok(());
                }
                Err(e) if attempt == max_attempts => {
                    warn!(
                        attempts = max_attempts,
                        error = %e,
                        "Server failed to start"
                    );
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
