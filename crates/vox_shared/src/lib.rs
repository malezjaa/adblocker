use std::path::PathBuf;

use clap::Parser;

pub mod config;
pub mod logger;
pub mod path;
pub mod pretty;
pub mod task;

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
