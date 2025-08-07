use std::pin::Pin;

use proto::api::registry::{
    registry_service_server::RegistryService,
    RegistryPullRequest,
    RegistryPullResponse,
    RegistryPushRequest,
    RegistryPushResponse,
};
use tokio_stream::Stream;
use tonic::{
    Request,
    Streaming,
    Result,
    Response,
    Status
};

#[derive(Default)]
pub struct LocalBackend {}

#[tonic::async_trait]
impl RegistryService for LocalBackend {
    type PullStream = Pin<Box<dyn Stream<Item = Result<RegistryPullResponse, Status>> + Send + 'static>>;

    async fn pull(
        &self,
        request: Request<RegistryPullRequest>,
    ) -> Result<Response<Self::PullStream>, Status> {
        _ = request;
        // Placeholder: return an empty stream
        let stream = tokio_stream::empty();
        Ok(Response::new(Box::pin(stream)))
    }

    async fn push(
        &self,
        request: Request<Streaming<RegistryPushRequest>>,
    ) -> Result<Response<RegistryPushResponse>, Status> {
        _ = request;
        todo!()
    }
}
