use crate::blocker::check_block;
use crate::firewall::external_dns::block_external_dns;
use crate::firewall::override_dns::override_default_dns;
use crate::state::State;
use anyhow::{Result, bail};
use hickory_proto::op::Message;
use hickory_resolver::net::{DnsError, NetError};
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tracing::info;

pub struct App {
  socket: UdpSocket,
  state: State,
}

impl App {
  pub async fn init(state: State) -> Result<Self> {
    let socket = UdpSocket::bind(state.socket()).await?;

    override_default_dns(state.socket(), state.secondary_name_server())?;
    block_external_dns(state.socket())?;

    Ok(Self { socket, state })
  }

  pub async fn run(&self) -> Result<()> {
    let mut buf = vec![0u8; 512];

    info!("DNS server running on {}", self.state.socket());

    loop {
      let (len, src) = match self.socket.recv_from(&mut buf).await {
        Ok(v) => v,
        Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
        Err(e) => return Err(e.into()),
      };

      let raw = buf[..len].to_vec();

      let start = Instant::now();
      let (blocked, response) = check_block(self.state.clone(), raw, false).await?;
      let elapsed = start.elapsed();

      self.socket.send_to(&response.to_vec()?, src).await?;
      self.state.spawn_query_record(&response, src, blocked, elapsed.as_millis() as i64);
    }
  }
}

pub async fn resolve_msg(msg: &Message, state: State) -> Result<Message> {
  let Some(query) = msg.queries.first() else { bail!("No name or record") };

  let mut response = msg.clone().into_response();

  match state.resolver().lookup(query.name.to_owned(), query.query_type).await {
    Ok(lookup) => {
      for record in lookup.answers() {
        response.add_answer(record.clone());
      }
    }
    Err(e) => match e {
      NetError::Dns(DnsError::NoRecordsFound(no)) => {
        response.metadata.response_code = no.response_code;
      }
      _ => return Err(e.into()),
    },
  }

  Ok(response)
}
