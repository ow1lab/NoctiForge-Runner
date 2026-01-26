use std::{collections::HashMap, sync::Arc};

use proto::api::worker::{
    ExecuteRequest, ExecuteResponse, execute_response::Outcome,
    worker_service_server::WorkerService,
};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::{debug, info, instrument, warn};

use crate::{client::controlplane_client::ControlPlaneClient, worker::organizer::NativeWorker};

pub struct WorkerServer {
    function_worker: Arc<Mutex<NativeWorker>>,
    controlplane_client: ControlPlaneClient,
}

impl WorkerServer {
    pub fn new(function_worker: NativeWorker, controlplane_client: ControlPlaneClient) -> Self {
        debug!("Creating WorkerServer");
        Self {
            function_worker: Arc::new(Mutex::new(function_worker)),
            controlplane_client,
        }
    }
}

#[tonic::async_trait]
impl WorkerService for WorkerServer {
    #[instrument(skip(self, request), fields(action = %request.get_ref().action))]
    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        let req = request.into_inner();

        // TODO: This should come from some where i don't know yet.
        let instance_id = "urn::invoke::abc1234";

        info!(action = %req.action, "Executing request");

        debug!(action = %req.action, "Fetching digest from control plane");
        let digest = self
            .controlplane_client
            .get_digest(req.action.clone())
            .await
            .map_err(|e| {
                warn!(action = %req.action, error = %e, "Failed to communicate with control plane");
                Status::internal(format!("Failed to commnicate with controlplane: {:?}", e))
            })?;

        debug!(action = %req.action, digest = %digest, body_size = req.body.len(), "Executing function");
        let output = {
            let mut worker = self.function_worker.lock().await;
            worker.execute(digest, req.body, req.metadata, instance_id).await
        };

        info!(action = %req.action, "Execution completed successfully");

        let outcome = Some(match output {
            Ok(outcome) => outcome,
            Err(_) => Outcome::Problem(proto::api::worker::ProblemDetails {
                r#type: "internal_error".to_string(),
                detail: "Something went worng here".to_string(),
                instance: instance_id.to_string(),
                extensions: HashMap::new(),
            }),
        });

        Ok(Response::new(ExecuteResponse { outcome }))
    }
}
