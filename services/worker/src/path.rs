use anyhow::{Ok, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

pub fn get_dir_path(digest: &str) -> PathBuf {
    get_pkgs_dir().join(digest)
}

pub fn get_instence_path(instance_id: &str) -> PathBuf {
    get_run_dir().join(instance_id)
}

pub async fn copy_dir_all(src: PathBuf, dst: PathBuf) -> Result<()> {
    let mut stack = vec![(src, dst)];

    while let Some((src, dst)) = stack.pop() {
        fs::create_dir_all(&dst).await?;

        let mut entries = fs::read_dir(&src).await?;

        while let Some(entry) = entries.next_entry().await? {
            let ft = entry.file_type().await?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if ft.is_dir() {
                stack.push((src_path, dst_path));
            } else {
                fs::copy(src_path, dst_path).await?;
            }
        }
    }

    Ok(())
}

fn get_run_dir() -> PathBuf {
    get_root_dir_path().join("run")
}

fn get_root_dir_path() -> PathBuf {
    Path::new("/var/lib/noctiforge/native_worker").to_path_buf()
}

fn get_pkgs_dir() -> PathBuf {
    get_root_dir_path().join("pkgs")
}
