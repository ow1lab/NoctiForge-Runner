use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Ok, Result};
use libcontainer::syscall::Syscall;
use proto::api::worker::ExecuteSuccess;
use proto::api::worker::execute_response::Outcome;
use tokio::time::sleep;
use tonic::Code;
use tonic::Request;
use tracing::{debug, info, instrument, warn};
use url::Url;

use crate::{
    client::registry_clint::RegistryClient,
    worker::{
        container::{self},
        function_invocations::FunctionInvocations,
        spec::SysUserParms,
    },
};
use proto::api::action::{
    InvokeRequest, function_runner_service_client::FunctionRunnerServiceClient,
};

const SERVER_STARTUP_TIMEOUT_MS: u64 = 3000;
const SERVER_STARTUP_RETRY_INTERVAL_MS: u64 = 10;

pub struct Config {
    pub is_dev: bool,
}

pub struct NativeWorker {
    function_invocations: Arc<FunctionInvocations>,
    registry_service: RegistryClient,
    root_path: PathBuf,
    sysuser: SysUserParms,
}

impl NativeWorker {
    pub fn new(
        function_invocations: &Arc<FunctionInvocations>,
        registry_service: RegistryClient,
        root_path: PathBuf,
        syscall: &dyn Syscall,
        server_config: Config,
    ) -> Result<Self> {
        info!(is_dev = server_config.is_dev, "Creating NativeWorker");
        Ok(Self {
            function_invocations: function_invocations.clone(),
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
    pub async fn execute(
        &mut self,
        digest: String,
        body: Vec<u8>,
        metadata: HashMap<String, String>,
        instance_id: &str 
    ) -> Result<Outcome> {
        debug!("Executing function");


        let uri = self.get_available_handler_uri(digest.clone()).await?;

        debug!(uri = %uri, "Connecting to function handler");
        let mut client = FunctionRunnerServiceClient::connect(uri.to_string())
            .await
            .map_err(|e| {
                warn!(digest = %digest, error = %e, "Failed to connect to function handler");
                e
            })?;

        let result = match client
            .invoke(Request::new(InvokeRequest {
                payload: body,
                metadata,
            }))
            .await
        {
            std::result::Result::Ok(resp) => {
                debug!(digest = %digest, "Function execution completed");

                match resp.into_inner().result {
                    Some(proto::api::action::invoke_result::Result::Success(e)) => {
                        Outcome::Success(ExecuteSuccess { body: e.output })
                    }
                    Some(proto::api::action::invoke_result::Result::Problem(p)) => {
                        Outcome::Problem(proto::api::worker::ProblemDetails {
                            r#type: p.r#type,
                            detail: p.detail,
                            instance: instance_id.to_string(),
                            extensions: HashMap::new(),
                        })
                    }
                    None => Outcome::Problem(proto::api::worker::ProblemDetails {
                        r#type: "missing_result".to_string(),
                        detail: "invoke returned no result".to_string(),
                        instance: instance_id.to_string(),
                        extensions: HashMap::new(),
                    }),
                }
            }

            Err(err) => {
                let problem = match err.code() {
                    Code::InvalidArgument => proto::api::worker::ProblemDetails {
                        r#type: "invalid_argument".to_string(),
                        detail: err.message().to_string(),
                        instance: instance_id.to_string(),
                        extensions: HashMap::new(),
                    },
                    Code::DataLoss => proto::api::worker::ProblemDetails {
                        r#type: "serialization_error".to_string(),
                        detail: err.message().to_string(),
                        instance: instance_id.to_string(),
                        extensions: HashMap::new(),
                    },
                    Code::Internal => proto::api::worker::ProblemDetails {
                        r#type: "internal_error".to_string(),
                        detail: err.message().to_string(),
                        instance: instance_id.to_string(),
                        extensions: HashMap::new(),
                    },
                    _ => proto::api::worker::ProblemDetails {
                        r#type: "invoke_failed".to_string(),
                        detail: err.message().to_string(),
                        instance: instance_id.to_string(), 
                        extensions: HashMap::new(),
                    },
                };

                Outcome::Problem(problem)
            }
        };

        Ok(result)
    }

    async fn get_available_handler_uri(&mut self, digest: String) -> Result<Url> {
        // TODO: This is a workaround i don't like as there could be a very low way that this
        // fails.
        let short_digest = &digest[..16];

        let url = if let Some(invocation) = self.function_invocations.get(short_digest).await {
            info!("Loading existing function");
            {
                let inv = invocation.lock().await;
                inv.url.clone()
            }
        } else {
            info!("Creating new function");
            let dir_path = self.registry_service.get_tar_by_digest(&digest).await?;

            let proc = Arc::new(
                container::ProccesContainer::new(
                    short_digest,
                    dir_path,
                    self.root_path.clone(),
                    &self.sysuser,
                )
                .await?,
            );

            let url = proc.get_url()?;
            self.function_invocations
                .insert(short_digest.to_string(), url.clone())
                .await;
            url
        };

        self.wait_for_server_ready(&url).await?;
        Ok(url)
    }

    async fn wait_for_server_ready(&self, url: &Url) -> Result<()> {
        let max_attempts = (SERVER_STARTUP_TIMEOUT_MS / SERVER_STARTUP_RETRY_INTERVAL_MS) as u32;
        let retry_interval = Duration::from_millis(SERVER_STARTUP_RETRY_INTERVAL_MS);

        let uri = url.to_string();
        debug!(max_attempts, uri, "Waiting for server to be ready");

        for attempt in 1..=max_attempts {
            match FunctionRunnerServiceClient::connect(uri.clone()).await {
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
