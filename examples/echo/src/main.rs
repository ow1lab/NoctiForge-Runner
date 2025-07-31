use sdk::Status;

async fn handler() -> Result<String, Status> {
    Ok("Hello, World!".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    sdk::start(handler).await
} 
