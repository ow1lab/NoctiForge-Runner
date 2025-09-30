use std::path::Path;

use proto::api::controlplane::{
    control_plane_service_server::ControlPlaneService,
    GetDigestByNameRequest,
    GetDigestByNameResponse, SetDigestToNameRequest, SetDigestToNameResponse
};
use tonic::{Request, Response, Status};

use crate::services::DigestService;

pub struct ControlPlane {
    digest_service: DigestService,
}

impl ControlPlane {
    pub async fn new(db_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            digest_service: DigestService::new(db_path).await?,
        })
    }
}

#[tonic::async_trait]
impl ControlPlaneService for ControlPlane {
    async fn get_digest_by_name(
        &self,
        request: Request<GetDigestByNameRequest>
    ) -> Result<Response<GetDigestByNameResponse>, Status> {
        let req = request.into_inner();
        self.digest_service.get_digest_by_name(&req.key).await
    }

    async fn set_digest_to_name(
        &self,
        request: Request<SetDigestToNameRequest>
    ) -> Result<Response<SetDigestToNameResponse>, Status> {
        let req = request.into_inner();
        self.digest_service.set_digest_by_name(&req.key, &req.digest).await
    }
}
