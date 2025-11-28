use std::path::PathBuf;

use anyhow::{Context, Result};
use libcontainer::{
    container::{builder::ContainerBuilder, Container},
    syscall::{syscall::SyscallType},
};
use tokio::{fs::{DirBuilder, File}, io::{AsyncWriteExt, BufWriter}};
use url::Url;
use uuid::Uuid;
use crate::{path::get_instence_path, worker::spec::{get_rootless, SysUserParms}};


pub struct ProccesContainer {
    container: Container,
    instance_id: String,
    sock_path: Url,
}

impl ProccesContainer {
    pub async fn new(root_path: PathBuf, handle_bin: PathBuf, sys_user: &SysUserParms) -> Result<Self> {
        _ = handle_bin;

        let instance_id = Uuid::new_v4().to_owned().to_string();
        let rootfs = Self::create_rootfs(&instance_id, sys_user).await?;

        let container = ContainerBuilder::new(instance_id.clone(), SyscallType::default())
            .with_root_path(root_path).expect("invalid root path")
            .as_init(rootfs)
            .with_systemd(false)
            .build()?;

        // TODO: make this inte instance path
        let sock_path = format!("unix:///run/noctiforge/{}/app.sock", instance_id);
        let url = Url::parse(&sock_path)?;

        Ok(Self {
            container,
            instance_id,
            sock_path: url,
        })
    }

    async fn create_rootfs(instance_id: &str, sys_user: &SysUserParms) -> Result<PathBuf> {
        let path = PathBuf::from(get_instence_path(instance_id));

        if path.exists() {
            anyhow::bail!("Root filesystem path already exists: {}", path.display());
        }

        DirBuilder::new().create(&path).await?;

        let spec = get_rootless(sys_user)?;

        // Create Spec
        let file = File::create(path.join("config.json")).await?;
        let mut writer = BufWriter::new(file);
        let json_bytes = serde_json::to_vec_pretty(&spec)?;
        writer.write_all(&json_bytes).await?;
        writer.flush().await?;

        DirBuilder::new().create(&path.join("rootfs")).await?;

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
        let path = PathBuf::from(get_instence_path(&self.instance_id));
        if path.exists() {
            tokio::fs::remove_dir_all(&path)
                .await
                .context("Failed to remove rootfs directory")?;
        }
        Ok(())
    }
}
