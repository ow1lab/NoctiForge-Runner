use anyhow::Result;
use proto::api::worker::worker_service_client::WorkerServiceClient;
use proto::api::worker::ExecuteRequest;
use tracing::{info, debug, error};

pub async fn run(key: String, body: String) -> Result<()> {
    info!("Triggering action: '{}'", key);
    debug!("Request body: {}", body);

    // Connect to the worker service
    let mut client = match WorkerServiceClient::connect("http://[::1]:50003").await {
        Ok(c) => {
            debug!("Connected to WorkerService");
            c
        }
        Err(e) => {
            error!("Failed to connect to WorkerService: {}", e);
            return Err(e.into());
        }
    };

    let request = tonic::Request::new(ExecuteRequest {
        action: key.clone(),
        body,
    });

    info!("Sending ExecuteRequest to worker");
    let response = match client.execute(request).await {
        Ok(resp) => {
            debug!("Received response from worker");
            resp
        }
        Err(e) => {
            error!("Worker execute call failed: {}", e);
            return Err(e.into());
        }
    };

    let output = response.into_inner().resp;
    info!("Worker executed action '{}', output:", key);
    println!("{output}");

    Ok(())
}

