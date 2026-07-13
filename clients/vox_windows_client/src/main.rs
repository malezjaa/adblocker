use anyhow::Result;
use clap::Parser;
use hickory_client::{
  client::Client,
  proto::{runtime::TokioRuntimeProvider, udp::UdpClientStream},
};
use tokio::{signal::ctrl_c, spawn};
use tracing::{error, warn};
use vox_dns::server_health::check_server_health;
use vox_shared::{SharedCli, logger::setup_logger, task::named_task, win_client_home};

use crate::{config::WinClientConfig, win_divert::WinDivert};

pub mod config;
pub mod win_divert;

#[tokio::main]
async fn main() {
  let cli = SharedCli::parse();
  setup_logger(cli.verbose);

  if let Err(err) = run().await {
    error!("{err:?}")
  }
}

async fn run() -> Result<()> {
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
  spawn(named_task("WinDivert", win_divert.start_redirects(client)));

  ctrl_c().await?;
  Ok(())
}
