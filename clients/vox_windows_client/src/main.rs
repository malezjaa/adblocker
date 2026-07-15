use anyhow::Result;
use clap::Parser;
use tokio::{runtime::Runtime, signal::ctrl_c, spawn, sync::oneshot::Receiver};
use vox_shared::{SharedCli, logger::setup_logger, task::named_task, win_client_home};

use crate::{config::WinClientConfig, upstream::UpstreamClient, win_divert::WinDivert};

pub mod config;
pub mod upstream;
pub mod win_divert;

#[cfg(windows)]
mod service;

fn main() -> Result<()> {
  let cli = SharedCli::parse();
  if cli.service {
    #[cfg(windows)]
    return service::dispatch();
    #[cfg(not(windows))]
    anyhow::bail!("--service is only supported on Windows");
  }

  setup_logger(cli.verbose);
  Runtime::new()?.block_on(run(None))
}

pub(crate) async fn run(shutdown: Option<Receiver<()>>) -> Result<()> {
  let config = WinClientConfig::from_file(win_client_home().join("config.toml"))?;
  let upstream = UpstreamClient::connect(&config).await?;

  let win_divert = WinDivert::new(config)?;
  let divert_task = spawn(named_task("WinDivert", win_divert.start_redirects(upstream)));

  match shutdown {
    Some(rx) => {
      let _ = rx.await;
    }
    None => ctrl_c().await?,
  }
  divert_task.abort();
  Ok(())
}
