use std::path::{Path, PathBuf};

pub fn get_dir_path(digest: &str) -> PathBuf {
    get_pkgs_dir().join(digest)
}

pub fn get_instence_path(instance_id: &str) -> PathBuf {
    get_run_dir().join(instance_id)
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
