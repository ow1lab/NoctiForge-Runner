use std::path::Path;

use proto::api::controlplane::{
    GetDigestByNameRequest, GetDigestByNameResponse, SetDigestToNameRequest,
    SetDigestToNameResponse, control_plane_service_server::ControlPlaneService,
};
use tonic::{Request, Response, Status};
use tracing::{debug, info, instrument};

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
    #[instrument(
        name = "Get digest to name",
        skip(self, request),
        fields(key = %request.get_ref().key)
    )]
    async fn get_digest_by_name(
        &self,
        request: Request<GetDigestByNameRequest>,
    ) -> Result<Response<GetDigestByNameResponse>, Status> {
        let req = request.into_inner();
        debug!(
            key = %req.key,
            "Received request to set digest"
        );
        let result = self.digest_service.get_digest_by_name(&req.key).await;
        
        match &result {
            Ok(_) => info!(key = %req.key, "Successfully retrieved digest"),
            Err(e) => debug!(key = %req.key, status = ?e.code(), "Failed to retrieve digest"),
        }
        
        result
    }

    #[instrument(
        name = "Set digest to name",
        skip(self, request),
        fields(key = %request.get_ref().key, digest_length = request.get_ref().digest.len())
    )]
    async fn set_digest_to_name(
        &self,
        request: Request<SetDigestToNameRequest>,
    ) -> Result<Response<SetDigestToNameResponse>, Status> {
        let req = request.into_inner();
        debug!(
            key = %req.key,
            digest_length = req.digest.len(),
            "Received request to set digest"
        );
        let result = self.digest_service
            .set_digest_by_name(&req.key, &req.digest)
            .await;

        match &result {
            Ok(_) => info!(key = %req.key, "Successfully set digest"),
            Err(e) => debug!(key = %req.key, status = ?e.code(), "Failed to set digest"),
        }

        result
    }
}
