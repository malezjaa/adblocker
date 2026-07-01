use std::path::PathBuf;

pub mod config;
pub mod logger;
pub mod task;

pub fn home_dir() -> PathBuf {
  dirs::home_dir().unwrap().join("vox")
}

pub fn win_client_home() -> PathBuf {
  dirs::home_dir().unwrap().join("vox_windows_client")
}
