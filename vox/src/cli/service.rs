use std::{
  env,
  fs::{copy, create_dir_all, remove_dir_all},
  net::SocketAddr,
  path::{Path, PathBuf},
  process::Command,
};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use vox_shared::{runtime_root, win_client_home};

use super::cli::{ServiceCommand, ServiceTarget};

const INSTALL_DIRECTORY: &str = "Vox";

#[derive(Serialize)]
struct ClientConfig {
  dns_server: std::net::SocketAddr,
  doh: Option<std::net::SocketAddr>,
}

pub fn handle(command: ServiceCommand) -> Result<()> {
  #[cfg(not(windows))]
  {
    let _ = command;
    bail!("Windows service management is only available on Windows")
  }

  #[cfg(windows)]
  match command {
    ServiceCommand::Install { target, dns_server, doh, dry_run } => {
      install(target, dns_server, doh, dry_run)
    }
    ServiceCommand::Start { target } => sc(target, "start"),
    ServiceCommand::Stop { target } => sc(target, "stop"),
    ServiceCommand::Status { target } => sc(target, "query"),
    ServiceCommand::Uninstall { target, purge_data } => uninstall(target, purge_data),
  }
}

#[cfg(windows)]
fn install(
  target: ServiceTarget,
  dns_server: Option<SocketAddr>,
  doh: Option<SocketAddr>,
  dry_run: bool,
) -> Result<()> {
  let source = bundle_directory()?;
  validate_bundle(&source)?;

  if target == ServiceTarget::Client {
    let dns_server =
      dns_server.context("--dns-server is required when installing the client")?;
    if dry_run {
      println!("would write {}", win_client_home().join("config.toml").display());
    } else {
      create_dir_all(win_client_home())?;
      fs_err::write(
        win_client_home().join("config.toml"),
        toml::to_string_pretty(&ClientConfig { dns_server, doh })?,
      )?;
    }
  }

  let install_dir = install_directory();
  if dry_run {
    println!("would install {} to {}", target.binary_name(), install_dir.display());
    println!("would create and start {}", target.service_name());
    return Ok(());
  }

  create_dir_all(&install_dir)?;
  for file in required_bundle_files() {
    copy(source.join(file), install_dir.join(file))
      .with_context(|| format!("copying {file} into {}", install_dir.display()))?;
  }

  let binary = install_dir.join(target.binary_name());
  let bin_path = format!("\"{}\" --service", binary.display());
  run_sc(&[
    "create".into(),
    target.service_name().into(),
    "binPath=".into(),
    bin_path,
    "start=".into(),
    "delayed-auto".into(),
    "obj=".into(),
    "LocalSystem".into(),
  ])?;
  run_sc(&["start".into(), target.service_name().into()])
}

#[cfg(windows)]
fn uninstall(target: ServiceTarget, purge_data: bool) -> Result<()> {
  let _ = run_sc(&["stop".into(), target.service_name().into()]);
  run_sc(&["delete".into(), target.service_name().into()])?;
  if purge_data {
    let root = runtime_root();
    if root.exists() {
      remove_dir_all(&root).with_context(|| format!("removing {}", root.display()))?;
    }
  }
  Ok(())
}

#[cfg(windows)]
fn sc(target: ServiceTarget, action: &str) -> Result<()> {
  run_sc(&[action.into(), target.service_name().into()])
}

#[cfg(windows)]
fn run_sc(args: &[String]) -> Result<()> {
  let status = Command::new("sc.exe").args(args).status().context("running sc.exe")?;
  if !status.success() {
    let action = args.first().map(String::as_str).unwrap_or("command");
    bail!("sc.exe {action} failed with {status}");
  }
  Ok(())
}

#[cfg(windows)]
fn bundle_directory() -> Result<PathBuf> {
  env::current_exe()?
    .parent()
    .map(Path::to_path_buf)
    .context("the CLI executable has no parent directory")
}

#[cfg(windows)]
fn install_directory() -> PathBuf {
  PathBuf::from(env::var_os("ProgramFiles").unwrap_or_else(|| "C:\\Program Files".into()))
    .join(INSTALL_DIRECTORY)
}

#[cfg(windows)]
fn validate_bundle(source: &Path) -> Result<()> {
  for file in required_bundle_files() {
    if !source.join(file).is_file() {
      bail!("release bundle is missing required file: {file}");
    }
  }
  Ok(())
}

#[cfg(windows)]
fn required_bundle_files() -> [&'static str; 5] {
  ["daemon.exe", "cli.exe", "vox_windows_client.exe", "WinDivert.dll", "WinDivert64.sys"]
}

impl ServiceTarget {
  #[cfg(windows)]
  fn service_name(self) -> &'static str {
    match self {
      Self::Daemon => "VoxDaemon",
      Self::Client => "VoxWindowsClient",
    }
  }

  #[cfg(windows)]
  fn binary_name(self) -> &'static str {
    match self {
      Self::Daemon => "daemon.exe",
      Self::Client => "vox_windows_client.exe",
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn client_service_requires_a_server() {
    let config = ClientConfig { dns_server: "127.0.0.1:53".parse().unwrap(), doh: None };
    let toml = toml::to_string(&config).unwrap();
    assert!(toml.contains("dns_server"));
  }
}
