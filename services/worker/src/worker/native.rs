use crate::worker::FunctionWorker;
use anyhow::Result;
use mktemp::Temp;

pub struct NativeWorker {
}

impl NativeWorker {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl FunctionWorker for NativeWorker {
    fn execute(&self, package_digest: String, body: String) -> Result<(), Box<dyn std::error::Error>> {
        // let temp_path = Temp::new_dir()?;
        // println!("Temp path: {:?}", temp_path);
        println!("Running in Docker with package: {}", package_digest);
        println!("Body: {}", body);
        Ok(())
    }
}
