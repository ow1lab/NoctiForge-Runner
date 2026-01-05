use sdk::Problem;

#[derive(serde::Deserialize)]
struct Request {
    name: String,
}

async fn handler(req: Request) -> Result<String, Problem> {
    if req.name.is_empty() {
        return Err(Problem {
            r#type: "user_echo/empty_name".to_string(),
            detail: "name is empty".to_string(),
        });
    }

    Ok(format!("Hello, {}!", req.name))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    sdk::start(handler).await
} 
