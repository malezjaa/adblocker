use clap::Parser;
use std::path::PathBuf;

pub mod config;
pub mod logger;
pub mod task;
pub mod path;
pub mod pretty;

pub fn home_dir() -> PathBuf {
  dirs::home_dir().unwrap().join("vox")
}

pub fn win_client_home() -> PathBuf {
  dirs::home_dir().unwrap().join("vox_windows_client")
}

#[derive(Parser, Debug)]
pub struct SharedCli {
  #[arg(short, long)]
  pub verbose: bool,
}
