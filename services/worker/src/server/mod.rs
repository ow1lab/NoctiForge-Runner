use std::sync::Arc;

use proto::api::worker::{worker_service_server::WorkerService, ExecuteRequest, ExecuteResponse};
use tonic::{Request, Response, Status};

use crate::{controlplane_client::ControlPlaneClient, worker::FunctionWorker};

pub struct WorkerServer {
    function_worker: Arc<dyn FunctionWorker + Send + Sync>,
    controlplane_client: ControlPlaneClient,
}

impl WorkerServer {
    pub fn new(function_worker: Arc<dyn FunctionWorker + Send + Sync>, controlplane_client: ControlPlaneClient) -> Self {
        Self { function_worker, controlplane_client}
    }
}

#[tonic::async_trait]
impl WorkerService for WorkerServer {
    async fn execute(
        &self,
        request: Request<ExecuteRequest>
    ) -> Result<Response<ExecuteResponse>, Status> {
       let req = request.into_inner();

       println!("Getting digest from key {}", req.action);
       let digits = self.controlplane_client.get_digest(req.action)
           .await
           .map_err(|e| Status::internal(format!("Failed to commnicate with controlplane: {:?}", e)))?;


       println!("Executing handler");
       self.function_worker.execute(digits, req.body).map_err(|e| Status::internal(format!("Execution failed: {:?}", e)))?;
        Ok(Response::new(ExecuteResponse{
            status: "Ok".to_string(),
            resp: "Anwser".to_string()
        }))
    }
}
