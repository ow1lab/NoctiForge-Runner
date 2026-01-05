use sdk::Problem;

#[derive(serde::Deserialize)]
struct Request {
    name: String,
}

async fn handler(req: Request) -> Result<String, Problem> {
    Ok(format!("Hello, {}!", req.name))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    sdk::start(handler).await
} 
