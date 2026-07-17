use std::net::{IpAddr, SocketAddr};

use anyhow::{Context, Result, bail};
use hickory_client::{
  client::Client,
  proto::{runtime::TokioRuntimeProvider, udp::UdpClientStream},
};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use tokio::{net::lookup_host, spawn};
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
    if let Some(doh) = config.doh.as_deref() {
      let doh = doh_connection(doh)?;
      let mut client = reqwest::Client::builder().no_proxy();
      match doh.resolve {
        DoHResolve::FixedAddress(addr) => {
          client = client.resolve(DOH_SERVER_NAME, addr);
        }
        DoHResolve::Hostname { hostname, port } => {
          let addresses = lookup_host((hostname.as_str(), port))
            .await
            .with_context(|| format!("resolving DNS-over-HTTPS hostname `{hostname}`"))?
            .collect::<Vec<_>>();
          if addresses.is_empty() {
            bail!("DNS-over-HTTPS hostname `{hostname}` resolved to no addresses");
          }
          client = client.resolve_to_addrs(&hostname, &addresses);
        }
        DoHResolve::None => {}
      }
      let client = client.build().context("creating DNS-over-HTTPS client")?;

      return Ok(Self::DoH { client, endpoint: doh.endpoint });
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

struct DoHConnection {
  endpoint: String,
  resolve: DoHResolve,
}

enum DoHResolve {
  FixedAddress(SocketAddr),
  Hostname { hostname: String, port: u16 },
  None,
}

fn doh_connection(server: &str) -> Result<DoHConnection> {
  if let Ok(addr) = server.parse::<SocketAddr>() {
    return Ok(DoHConnection {
      endpoint: format!("https://{DOH_SERVER_NAME}:{}/dns-query", addr.port()),
      resolve: DoHResolve::FixedAddress(addr),
    });
  }

  let endpoint = if server.contains("://") {
    server.to_owned()
  } else {
    format!("https://{server}/dns-query")
  };
  let endpoint = reqwest::Url::parse(&endpoint)
    .with_context(|| format!("parsing DNS-over-HTTPS endpoint `{server}`"))?;

  if endpoint.scheme() != "https" {
    bail!("DNS-over-HTTPS endpoint must use HTTPS: `{server}`");
  }
  if endpoint.host_str().is_none() {
    bail!("DNS-over-HTTPS endpoint must include a hostname: `{server}`");
  }

  let resolve = match endpoint.host_str().unwrap().parse::<IpAddr>() {
    Ok(_) => DoHResolve::None,
    Err(_) => DoHResolve::Hostname {
      hostname: endpoint.host_str().unwrap().to_owned(),
      port: endpoint.port_or_known_default().unwrap(),
    },
  };

  Ok(DoHConnection { endpoint: endpoint.into(), resolve })
}

#[cfg(test)]
mod tests {
  use super::{DOH_SERVER_NAME, DoHResolve, doh_connection};

  #[test]
  fn uses_a_doh_hostname_for_the_endpoint_and_tls() {
    let connection = doh_connection("doh.example.com").unwrap();

    assert_eq!(connection.endpoint, "https://doh.example.com/dns-query");
    assert!(matches!(
      connection.resolve,
      DoHResolve::Hostname { hostname, port: 443 }
        if hostname == "doh.example.com"
    ));
  }

  #[test]
  fn supports_ip_address_configuration() {
    let connection = doh_connection("192.0.2.10:8443").unwrap();

    assert_eq!(connection.endpoint, format!("https://{DOH_SERVER_NAME}:8443/dns-query"));
    assert!(matches!(
      connection.resolve,
      DoHResolve::FixedAddress(addr) if addr.to_string() == "192.0.2.10:8443"
    ));
  }

  #[test]
  fn rejects_insecure_doh_endpoint() {
    assert!(doh_connection("http://doh.example.com").is_err());
  }
}
