use std::{
  env,
  path::{Path, PathBuf},
  process::Command,
  thread::sleep,
  time::Duration,
};

use anyhow::{Context, Result, bail};
use fs_err::{copy, create_dir_all, remove_dir_all};
use vox_shared::{
  pretty::{print_field, print_message, print_separator, print_success},
  runtime_root, win_client_home,
};
#[cfg(windows)]
use windows_service::{
  Error as WindowsServiceError,
  service::{ServiceAccess, ServiceState},
  service_manager::{ServiceManager, ServiceManagerAccess},
};
use yansi::Paint;

use super::cli::{ServiceCommand, ServiceTarget};

const INSTALL_DIRECTORY: &str = "Vox";

pub fn handle(command: ServiceCommand) -> Result<()> {
  #[cfg(not(windows))]
  {
    let _ = command;
    bail!("Windows service management is only available on Windows")
  }

  #[cfg(windows)]
  match command {
    ServiceCommand::Install { target, dry_run } => install(target, dry_run),
    ServiceCommand::Start { target } => start(target),
    ServiceCommand::Stop { target } => stop(target),
    ServiceCommand::Restart { target } => restart(target),
    ServiceCommand::Status { target } => status(target),
    ServiceCommand::Uninstall { target, purge_data } => uninstall(target, purge_data),
  }
}

#[cfg(windows)]
fn install(target: ServiceTarget, dry_run: bool) -> Result<()> {
  let source = bundle_directory()?;
  validate_bundle(&source)?;

  if target == ServiceTarget::Client {
    let win_config_path = win_client_home().join("config.toml");

    if !win_config_path.exists() {
      bail!(
        "Please create `{}` with `dns_server` set to the Vox server address. Optionally \
         you can set `doh`.",
        win_config_path.display()
      );
    }
  }

  let install_dir = install_directory();
  if dry_run {
    println!("would install {} to {}", target.binary_name(), install_dir.display());
    println!("would create and start {}", target.service_name());
    return Ok(());
  }

  create_dir_all(&install_dir)?;
  let services_to_restart = if bundle_needs_update(&source, &install_dir)? {
    stop_bundle_services()?
  } else {
    Vec::new()
  };

  for file in required_bundle_files() {
    let source_file = source.join(file);
    let destination_file = install_dir.join(file);
    if !files_match(&source_file, &destination_file)? {
      copy(&source_file, &destination_file)
        .with_context(|| format!("copying {file} into {}", install_dir.display()))?;
    }
  }

  let binary = install_dir.join(target.binary_name());
  let bin_path = format!("\"{}\" --service", binary.display());
  configure_service(target, bin_path)?;
  for service in services_to_restart {
    if service != target {
      start_service(service)?;
    }
  }
  start_service(target)?;
  print_success(&format!("{} service installed", target.display_name()));
  Ok(())
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
  print_success(&format!("{} service uninstalled", target.display_name()));
  Ok(())
}

#[cfg(windows)]
fn start(target: ServiceTarget) -> Result<()> {
  sc(target, "start")?;
  print_success(&format!("{} service started", target.display_name()));
  Ok(())
}

#[cfg(windows)]
fn stop(target: ServiceTarget) -> Result<()> {
  sc(target, "stop")?;
  print_success(&format!("{} service stopped", target.display_name()));
  Ok(())
}

#[cfg(windows)]
fn restart(target: ServiceTarget) -> Result<()> {
  stop_service(target)?;
  start_service(target)?;
  print_success(&format!("{} service restarted", target.display_name()));
  Ok(())
}

#[cfg(windows)]
fn status(target: ServiceTarget) -> Result<()> {
  let state = service_state(target)?
    .with_context(|| format!("{} service is not installed", target.display_name()))?;

  println!();
  print_message(&format!("{} service", target.display_name()));
  print_separator(30);
  print_field("Name:  ", target.service_name());
  print_field("Status:", pretty_service_state(state));
  print_separator(30);
  Ok(())
}

#[cfg(windows)]
fn pretty_service_state(state: ServiceState) -> String {
  match state {
    ServiceState::Running => "Running".green().bold().to_string(),
    ServiceState::Stopped => "Stopped".dim().to_string(),
    ServiceState::StartPending => "Starting".bright_yellow().to_string(),
    ServiceState::StopPending => "Stopping".bright_yellow().to_string(),
    ServiceState::ContinuePending => "Resuming".bright_yellow().to_string(),
    ServiceState::PausePending => "Pausing".bright_yellow().to_string(),
    ServiceState::Paused => "Paused".bright_yellow().to_string(),
  }
}

#[cfg(windows)]
fn sc(target: ServiceTarget, action: &str) -> Result<()> {
  run_sc(&[action.into(), target.service_name().into()])
}

#[cfg(windows)]
fn run_sc(args: &[String]) -> Result<()> {
  let output = Command::new("sc.exe").args(args).output().context("running sc.exe")?;
  if !output.status.success() {
    let action = args.first().map(String::as_str).unwrap_or("command");
    let message = format!(
      "{}{}",
      String::from_utf8_lossy(&output.stdout),
      String::from_utf8_lossy(&output.stderr)
    );
    bail!("service {action} failed: {}", message.trim());
  }
  Ok(())
}

#[cfg(windows)]
fn configure_service(target: ServiceTarget, bin_path: String) -> Result<()> {
  let args = vec![
    if service_state(target)?.is_some() { "config" } else { "create" }.into(),
    target.service_name().into(),
    "binPath=".into(),
    bin_path,
    "start=".into(),
    "delayed-auto".into(),
    "obj=".into(),
    "LocalSystem".into(),
  ];
  run_sc(&args)
}

#[cfg(windows)]
fn bundle_needs_update(source: &Path, destination: &Path) -> Result<bool> {
  for file in required_bundle_files() {
    if !files_match(&source.join(file), &destination.join(file))? {
      return Ok(true);
    }
  }
  Ok(false)
}

#[cfg(windows)]
fn files_match(source: &Path, destination: &Path) -> Result<bool> {
  if !destination.is_file() {
    return Ok(false);
  }
  if fs_err::metadata(source)?.len() != fs_err::metadata(destination)?.len() {
    return Ok(false);
  }
  Ok(fs_err::read(source)? == fs_err::read(destination)?)
}

#[cfg(windows)]
fn stop_bundle_services() -> Result<Vec<ServiceTarget>> {
  let mut services_to_restart = Vec::new();
  for target in [ServiceTarget::Daemon, ServiceTarget::Client] {
    if matches!(service_state(target)?, Some(state) if state != ServiceState::Stopped) {
      stop_service(target)?;
      services_to_restart.push(target);
    }
  }
  Ok(services_to_restart)
}

#[cfg(windows)]
fn stop_service(target: ServiceTarget) -> Result<()> {
  for _ in 0..300 {
    match service_state(target)? {
      None | Some(ServiceState::Stopped) => return Ok(()),
      Some(ServiceState::StartPending | ServiceState::StopPending) => {
        sleep(Duration::from_millis(100));
      }
      Some(_) => {
        run_sc(&["stop".into(), target.service_name().into()])?;
        sleep(Duration::from_millis(100));
      }
    }
  }
  bail!("timed out stopping {}", target.service_name())
}

#[cfg(windows)]
fn start_service(target: ServiceTarget) -> Result<()> {
  for _ in 0..300 {
    match service_state(target)? {
      Some(ServiceState::Running) => return Ok(()),
      Some(ServiceState::StartPending | ServiceState::StopPending) => {
        sleep(Duration::from_millis(100));
      }
      None => bail!("{} does not exist", target.service_name()),
      Some(_) => {
        run_sc(&["start".into(), target.service_name().into()])?;
        sleep(Duration::from_millis(100));
      }
    }
  }
  bail!("timed out starting {}", target.service_name())
}

#[cfg(windows)]
fn service_state(target: ServiceTarget) -> Result<Option<ServiceState>> {
  let manager =
    ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
      .context("connecting to the Windows service manager")?;
  match manager.open_service(target.service_name(), ServiceAccess::QUERY_STATUS) {
    Ok(service) => Ok(Some(service.query_status()?.current_state)),
    Err(WindowsServiceError::Winapi(error)) if error.raw_os_error() == Some(1060) => {
      Ok(None)
    }
    Err(error) => Err(error).context("opening Windows service"),
  }
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
  fn display_name(self) -> &'static str {
    match self {
      Self::Daemon => "Daemon",
      Self::Client => "Client",
    }
  }

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
