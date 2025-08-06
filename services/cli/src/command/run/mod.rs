use anyhow::Result;
use serde::Deserialize;
use std::fs::read_to_string;
use std::io::{
    Error,
    ErrorKind,
};

mod custom;

#[derive(Debug, Deserialize)]
struct NoctiConfig {
    project: Project,
    run: RunType,
}

#[derive(Debug, Deserialize)]
struct Project {
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RunType {
    Custom(custom::CustomRun),
}

pub async fn run(path: String) -> Result<()> {
    let file_path = path.clone() + "/Nocti.toml";

    let content = read_to_string(&file_path).map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            Error::new(
                ErrorKind::NotFound,
                format!("File not found: {}. Please check if it exists.", path.clone() + "/Nocti.toml"),
            )
        } else {
            e
        }
    })?;

    let cfg: NoctiConfig = toml::from_str(&content).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Failed to parse Nocti.toml: {}", e),
        )
    })?;

    println!("Project Name: {}", cfg.project.name);

    match cfg.run {
        RunType::Custom(custom) => {
            custom::process(custom, &path).await?;
        }
    }

    return Ok(())
}
