use crate::config::WinClientConfig;
use anyhow::{Result, bail};
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use tracing::error;
use vox_shared::logger::setup_logger;
use vox_shared::{home_dir, win_client_home};

use hickory_client::client::{Client, ClientHandle};
use hickory_client::proto::op::Query;
use hickory_client::proto::rr::{DNSClass, Name, RecordType};
use hickory_client::proto::runtime::TokioRuntimeProvider;
use hickory_client::proto::udp::UdpClientStream;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use vox_dns::server_health::check_server_health;

pub mod config;
pub mod win_divert;

#[tokio::main]
async fn main() {
  if let Err(err) = run().await {
    error!("{err:?}")
  }
}

async fn run() -> Result<()> {
  setup_logger(false);
  let config = WinClientConfig::from_file(win_client_home().join("config.toml"))?;
  check_server_health(&config.dns_server).await?;

  let stream =
    UdpClientStream::builder(config.dns_server, TokioRuntimeProvider::default()).build();
  let (mut client, bg) = Client::connect(stream).await?;
  tokio::spawn(bg);

  Ok(())
}
