use crate::application::pipeline::Pipeline;
use crate::blocklists::load_blocklists;
use crate::config::Config;
use crate::context::Context;
use crate::firewall::external_dns::block_external_dns;
use crate::middleware::MiddlewareResult;
use crate::middlewares::blocker::Blocker;
use anyhow::bail;
use fs_err::create_dir_all;
use hickory_proto::op::{Message, ResponseCode, UpdateMessage};
use hickory_proto::serialize::binary::{BinDecodable, BinEncodable};
use hickory_resolver::net::{DnsError, NetError};
use hickory_resolver::{config::*, *};
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tracing::{debug, info};

pub struct App {
  socket: Arc<UdpSocket>,
  pipeline: Pipeline,
  resolver: TokioResolver,
}

impl App {
  pub async fn init() -> anyhow::Result<Self> {
    let home_path = dirs::home_dir().unwrap().join("adb");
    let config = Config::from_file(home_path.join("config.toml"))?;
    let cache_dir = home_path.join("cache");

    if !cache_dir.exists() {
      create_dir_all(&cache_dir)?;
    }

    let socket = Arc::new(UdpSocket::bind(config.socket).await?);

    // block_external_dns(config.socket)?;

    let start = Instant::now();
    let rules = load_blocklists(config.blocklists, &cache_dir).await?;
    info!("loaded lists in {:.2?}", start.elapsed());

    let pipeline = Pipeline::new().add(Blocker::new(rules));

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

        // Get a new resolver with the google nameservers as the upstream recursive resolvers
        Resolver::tokio(ResolverConfig::udp_and_tcp(), ResolverOpts::default())
      }
    };

    Ok(Self {
      socket,
      pipeline,
      resolver,
    })
  }
  pub async fn run(&self) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 512];
    loop {
      let (len, src) = match self.socket.recv_from(&mut buf).await {
        Ok(v) => v,
        Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
        Err(e) => return Err(e.into()),
      };

      let mut ctx = Context::new(buf[..len].to_vec())?;
      debug!(request = ?ctx, from = %src);
      self.handle(&mut ctx, src).await?;
    }
  }

  async fn handle(&self, ctx: &mut Context, src: SocketAddr) -> anyhow::Result<()> {
    match self.pipeline.run(ctx).await {
      MiddlewareResult::Block => ctx.send_blocked(&self.socket, src).await?,
      MiddlewareResult::Respond(msg) => send_response(&self.socket, src, msg).await?,
      MiddlewareResult::Next => self.resolve(ctx, src).await?,
    }
    Ok(())
  }

  async fn resolve(&self, ctx: &mut Context, src: SocketAddr) -> anyhow::Result<()> {
    let Some(query) = ctx.query() else {
      bail!("No name or record")
    };

    let mut response = ctx.msg().clone().into_response();

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
) -> anyhow::Result<()> {
  socket.send_to(&msg.to_bytes()?, src).await?;
  Ok(())
}
