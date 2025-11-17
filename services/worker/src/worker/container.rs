use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use libcontainer::{
    container::{Container, builder::ContainerBuilder},
    syscall::syscall::SyscallType,
};
use tokio::fs::DirBuilder;
use url::Url;
use uuid::Uuid;

pub struct ProccesContainer {
    container: Container,
    instance_id: String,
    sock_path: Url,
}

impl ProccesContainer {
    pub async fn new(handle_bin: PathBuf) -> Result<Self> {
        let instance_id = Uuid::new_v4().to_owned().to_string();

        Self::create_rootfs(&instance_id).await?;

        let container = ContainerBuilder::new(instance_id.clone(), SyscallType::default())
            .with_root_path(format!("/run/noctiforge/{}", &instance_id))
            .expect("invalid root path")
            .as_init(handle_bin)
            .with_systemd(false)
            .build()?;

        let sock_path = format!("unix:///run/noctiforge/{}/app.sock", instance_id);
        let url = Url::parse(&sock_path)?;

        Ok(Self {
            container,
            instance_id,
            sock_path: url,
        })
    }

    fn get_root_path(instance_id: &str) -> String {
        format!("/run/noctiforge/{}", instance_id)
    }

    async fn create_rootfs(instance_id: &str) -> Result<PathBuf> {
        let path = PathBuf::from(Self::get_root_path(instance_id));

        if path.exists() {
            anyhow::bail!("Root filesystem path already exists: {}", path.display());
        }

        DirBuilder::new().create(&path).await?;

        Ok(path)
    }

    pub fn start(&mut self) -> Result<()> {
        self.container.start()?;
        Ok(())
    }

    pub fn get_url(&self) -> Url {
        self.sock_path.clone()
    }

    #[allow(dead_code)]
    pub async fn cleanup(&self) -> Result<()> {
        let path = PathBuf::from(Self::get_root_path(&self.instance_id));
        if path.exists() {
            tokio::fs::remove_dir_all(&path)
                .await
                .context("Failed to remove rootfs directory")?;
        }
        Ok(())
    }
}
