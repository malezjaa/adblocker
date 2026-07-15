use std::net::SocketAddr;

use anyhow::{Context, Result};
use hickory_client::{
  client::Client,
  proto::{runtime::TokioRuntimeProvider, udp::UdpClientStream},
};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use tokio::spawn;
use tracing::{error, warn};
use vox_dns::{dns_query::DnsQuery, server_health::check_server_health};

use crate::config::WinClientConfig;

const DNS_MESSAGE_CONTENT_TYPE: &str = "application/dns-message";
const DOH_SERVER_NAME: &str = "doh.local";

pub enum UpstreamClient {
  Plain(Client),
  DoH { client: reqwest::Client, endpoint: String },
}

impl UpstreamClient {
  pub async fn connect(config: &WinClientConfig) -> Result<Self> {
    if let Some(addr) = config.doh {
      let client = reqwest::Client::builder()
        .no_proxy()
        .resolve(DOH_SERVER_NAME, addr)
        .build()
        .context("creating DNS-over-HTTPS client")?;

      return Ok(Self::DoH { client, endpoint: doh_endpoint(addr) });
    }

    check_server_health(&config.dns_server).await?;

    let stream =
      UdpClientStream::builder(config.dns_server, TokioRuntimeProvider::default())
        .build();
    let (client, bg) = Client::connect(stream).await?;
    let bg_handle = spawn(bg);
    spawn(async move {
      match bg_handle.await {
        Ok(Ok(())) => warn!("hickory background exchange task exited cleanly"),
        Ok(Err(error)) => {
          error!("hickory background exchange task errored: {error:?}")
        }
        Err(error) => error!("hickory background exchange task panicked: {error:?}"),
      }
    });

    Ok(Self::Plain(client))
  }

  pub async fn send(&self, query: DnsQuery) -> Result<Vec<u8>> {
    match self {
      Self::Plain(client) => Ok(query.send(client).await?.to_vec()?),
      Self::DoH { client, endpoint } => {
        let request = query.into_message().to_vec()?;
        let response = client
          .post(endpoint)
          .header(CONTENT_TYPE, DNS_MESSAGE_CONTENT_TYPE)
          .header(ACCEPT, DNS_MESSAGE_CONTENT_TYPE)
          .body(request)
          .send()
          .await
          .context("sending DNS-over-HTTPS request")?
          .error_for_status()
          .context("DNS-over-HTTPS server returned an error status")?;

        let response =
          response.bytes().await.context("reading DNS-over-HTTPS response")?;
        Ok(response.to_vec())
      }
    }
  }
}

fn doh_endpoint(addr: SocketAddr) -> String {
  format!("https://{DOH_SERVER_NAME}:{}/dns-query", addr.port())
}
