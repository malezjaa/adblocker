use crate::blocker::{BlockLookup, BlockResult};
use anyhow::{bail, Result};
use hickory_proto::op::{Message, ResponseCode, UpdateMessage};
use hickory_proto::rr::rdata::{A, AAAA};
use hickory_proto::rr::{RData, Record, RecordType};
use hickory_proto::serialize::binary::{BinDecodable, BinEncodable};
use hickory_resolver::net::{DnsError, NetError};
use hickory_resolver::*;
use std::io::ErrorKind;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

pub struct App {
  socket: Arc<UdpSocket>,
  resolver: TokioResolver,
  tx: Sender<BlockLookup>,
}

impl App {
  pub async fn init(socket: SocketAddr, tx: Sender<BlockLookup>) -> Result<Self> {
    let socket = Arc::new(UdpSocket::bind(socket).await?);

    // block_external_dns(config.socket)?;

    let resolver = {
      // To make this independent, if targeting macOS, BSD, Linux, or Windows, we can use the system's configuration:
      #[cfg(any(unix, windows))]
      {
        use hickory_resolver::{net::runtime::TokioRuntimeProvider, TokioResolver};

        // use the system resolver configuration
        TokioResolver::builder(TokioRuntimeProvider::default())?
          .build()?
      }

      // For other operating systems, we can use one of the preconfigured definitions
      #[cfg(not(any(unix, windows)))]
      {
        // Directly reference the config types
        use hickory_resolver::{
          config::{ResolverConfig, ResolverOpts, GOOGLE},
          Resolver,
        };

        // Get a new resolver with the Google nameservers as the upstream recursive resolvers
        Resolver::tokio(ResolverConfig::udp_and_tcp(), ResolverOpts::default())
      }
    };

    Ok(Self {
      socket,
      resolver,
      tx,
    })
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
      let msg = Message::from_bytes(&raw)?;

      let (sender, rx) = oneshot::channel();

      let lookup = BlockLookup::new(
        msg.clone(),
        sender,
      );

      self.tx.send(lookup).await?;

      match rx.await? {
        BlockResult::Ok => self.resolve(&msg, src).await?,
        BlockResult::Block => self.block(&msg, &self.socket, src).await?,
      }
    }
  }

  async fn block(&self, msg: &Message, socket: &UdpSocket, src: SocketAddr) -> Result<()> {
    let mut response = Message::response(msg.id(), msg.op_code);

    response.add_queries(msg.queries.clone());

    for query in &msg.queries {
      let rdata = match query.query_type() {
        RecordType::A => Some(RData::A(A(Ipv4Addr::new(0, 0, 0, 0)))),
        RecordType::AAAA => {
          Some(RData::AAAA(AAAA(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0))))
        }
        _ => None,
      };

      if let Some(rdata) = rdata {
        let record = Record::from_rdata(query.name().clone(), 0, rdata);
        response.add_answer(record);
      }
    }

    if response.answers.is_empty() {
      response.metadata.response_code = ResponseCode::NXDomain;
    }

    let bytes = response.to_vec()?;
    socket.send_to(&bytes, src).await?;
    Ok(())
  }

  async fn resolve(&self, msg: &Message, src: SocketAddr) -> Result<()> {
    let Some(query) = msg.queries.first() else {
      bail!("No name or record")
    };

    let mut response = msg.clone().into_response();

    match self.resolver.lookup(query.name.to_owned(), query.query_type).await {
      Ok(lookup) => {
        for record in lookup.answers() {
          response.add_answer(record.clone());
        }
      }
      Err(e) => {
        match e {
          NetError::Dns(DnsError::NoRecordsFound(no)) => {
            response.metadata.response_code = no.response_code;
          }
          _ => return Err(e.into()),
        }
      }
    }

    send_response(&self.socket, src, response).await?;
    Ok(())
  }
}

pub async fn send_response(
  socket: &UdpSocket,
  src: SocketAddr,
  msg: Message,
) -> Result<()> {
  socket.send_to(&msg.to_bytes()?, src).await?;
  Ok(())
}
