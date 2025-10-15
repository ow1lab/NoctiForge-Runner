use anyhow::Result;

pub mod native;

pub trait FunctionWorker: Send + Sync {
    fn execute(&self, package_digest: String, body: String) -> Result<(), Box<dyn std::error::Error>>;
}
