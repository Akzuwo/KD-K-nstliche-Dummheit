pub mod engine;
pub mod markdown;
pub mod models;
pub mod store;
#[cfg(windows)]
pub mod updater;

use std::path::PathBuf;

pub fn data_root() -> PathBuf {
    std::env::var_os("KD_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            directories::BaseDirs::new()
                .expect("Windows-Benutzerverzeichnis fehlt")
                .data_local_dir()
                .join("KD Desktop")
        })
}

pub fn runtime_root() -> PathBuf {
    std::env::var_os("KD_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("runtime")
        })
}
