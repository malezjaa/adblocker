use crate::blocker::check_block;
use crate::firewall::override_dns::override_default_dns;
use crate::state::State;
use anyhow::{Result, bail};
use hickory_proto::op::Message;
use hickory_resolver::net::{DnsError, NetError};
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

pub struct App {
  socket: Arc<UdpSocket>,
  state: State,
}

impl App {
  pub async fn init(socket: SocketAddr, state: State) -> Result<Self> {
    let socket = Arc::new(UdpSocket::bind(socket).await?);

    override_default_dns(state.socket().await, state.secondary_name_server().await)?;
    // block_external_dns(config.socket)?;

    Ok(Self { socket, state })
  }

  pub async fn run(&self) -> Result<()> {
    let mut buf = vec![0u8; 512];

    loop {
      let (len, src) = match self.socket.recv_from(&mut buf).await {
        Ok(v) => v,
        Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
        Err(e) => return Err(e.into()),
      };

      let raw = buf[..len].to_vec();

      let (blocked, response) = check_block(self.state.clone(), raw, false).await?;
      self.socket.send_to(&response.to_vec()?, src).await?;

      self.state.spawn_query_record(&response, src, blocked);
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
