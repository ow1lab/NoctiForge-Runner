use std::path::{Path, PathBuf};

pub fn get_root_dir_path() -> PathBuf {
    Path::new("/var/lib/noctiforge").to_path_buf()
}

pub fn get_registry_dir_path() -> PathBuf {
    get_root_dir_path().join("registry")
}

pub fn get_registry_path(digest: &str) -> PathBuf {
    get_registry_dir_path().join(digest).with_extension("tar")
}
