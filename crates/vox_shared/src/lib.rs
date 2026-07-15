use std::path::PathBuf;

use clap::Parser;

pub mod config;
pub mod logger;
pub mod path;
pub mod pretty;
pub mod task;

pub fn home_dir() -> PathBuf {
  runtime_root().join("daemon")
}

pub fn win_client_home() -> PathBuf {
  runtime_root().join("client")
}

pub fn logs_dir() -> PathBuf {
  runtime_root().join("logs")
}

pub fn runtime_root() -> PathBuf {
  #[cfg(windows)]
  {
    windows_runtime_root(std::env::var_os("PROGRAMDATA").map(PathBuf::from))
  }

  #[cfg(not(windows))]
  {
    dirs::home_dir().expect("a home directory is required").join("vox")
  }
}

#[cfg(windows)]
fn windows_runtime_root(program_data: Option<PathBuf>) -> PathBuf {
  program_data.unwrap_or_else(|| PathBuf::from(r"C:\ProgramData")).join("Vox")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(windows)]
  #[test]
  fn windows_runtime_root_uses_program_data() {
    assert_eq!(
      windows_runtime_root(Some(PathBuf::from(r"D:\ProgramData"))),
      PathBuf::from(r"D:\ProgramData\Vox")
    );
  }
}

#[derive(Parser, Debug)]
pub struct SharedCli {
  #[arg(short, long)]
  pub verbose: bool,

  /// Internal switch used by the Windows Service Control Manager.
  #[arg(long, hide = true)]
  pub service: bool,
}
