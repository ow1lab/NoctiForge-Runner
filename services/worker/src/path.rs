use std::path::{PathBuf, Path};

pub fn get_root_dir_path() -> PathBuf {
    Path::new("/var/lib/noctiforge").to_path_buf()
}

pub fn get_cached_dir() -> PathBuf {
    get_root_dir_path().join("native_worker")
}

pub fn get_dir_path(digest: &str) -> PathBuf {
    get_cached_dir()
        .join(digest)
}
