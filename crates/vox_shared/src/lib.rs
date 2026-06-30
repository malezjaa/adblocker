use std::path::PathBuf;

pub mod config;
pub mod logger;
pub mod task;

pub fn home_dir() -> PathBuf {
  dirs::home_dir().unwrap().join("adb")
}