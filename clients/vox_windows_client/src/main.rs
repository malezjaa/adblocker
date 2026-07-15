use anyhow::Result;
use clap::Parser;
use hickory_client::{
  client::Client,
  proto::{runtime::TokioRuntimeProvider, udp::UdpClientStream},
};
use tokio::{runtime::Runtime, signal::ctrl_c, spawn, sync::oneshot::Receiver};
use tracing::{error, warn};
use vox_dns::server_health::check_server_health;
use vox_shared::{SharedCli, logger::setup_logger, task::named_task, win_client_home};

use crate::{config::WinClientConfig, win_divert::WinDivert};

pub mod config;
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
  check_server_health(&config.dns_server).await?;

  let stream =
    UdpClientStream::builder(config.dns_server, TokioRuntimeProvider::default()).build();
  let (client, bg) = Client::connect(stream).await?;
  let bg_handle = spawn(bg);
  spawn(async move {
    match bg_handle.await {
      Ok(Ok(())) => warn!("hickory background exchange task exited cleanly"),
      Ok(Err(e)) => error!("hickory background exchange task errored: {e:?}"),
      Err(e) => error!("hickory background exchange task panicked: {e:?}"),
    }
  });

  let win_divert = WinDivert::new(config)?;
  let divert_task = spawn(named_task("WinDivert", win_divert.start_redirects(client)));

  match shutdown {
    Some(rx) => {
      let _ = rx.await;
    }
    None => ctrl_c().await?,
  }
  divert_task.abort();
  Ok(())
}
