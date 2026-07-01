use crate::config::WinClientConfig;
use anyhow::Result;
use std::str::FromStr;
use tracing::error;
use vox_shared::logger::setup_logger;
use vox_shared::{home_dir, win_client_home};

use hickory_client::client::{Client, ClientHandle};
use hickory_client::proto::op::Query;
use hickory_client::proto::rr::{DNSClass, Name, RecordType};
use hickory_client::proto::runtime::TokioRuntimeProvider;
use hickory_client::proto::udp::UdpClientStream;
use vox_shared::dns_query::DnsQuery;
use vox_shared::edns::EDNSCode;

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

  let stream =
    UdpClientStream::builder(config.dns_server, TokioRuntimeProvider::default()).build();
  let (mut client, bg) = Client::connect(stream).await?;
  tokio::spawn(bg);

  let query =
    DnsQuery::new(Query::query(Name::from_str("example.com.")?, RecordType::A))?
      .send(&client)
      .await?;
  println!("{:?}", query);

  let query =
    DnsQuery::new(Query::query(Name::from_str("example.com.")?, RecordType::A))?
      .add_edns_option(EDNSCode::BlockOrigin, &[10, 20])
      .send(&client)
      .await?;
  println!("{:?}", query);

  Ok(())
}
