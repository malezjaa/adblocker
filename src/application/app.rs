use crate::application::pipeline::Pipeline;
use crate::blocklists::load_blocklists;
use crate::config::Config;
use crate::context::Context;
use crate::middleware::MiddlewareResult;
use crate::middlewares::blocker::Blocker;
use crate::response_cache::ResponseCache;
use fs_err::create_dir_all;
use hickory_proto::op::{Message, ResponseCode, UpdateMessage};
use hickory_proto::serialize::binary::{BinDecodable, BinEncodable};
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tracing::{debug, info};

pub struct App {
  socket: Arc<UdpSocket>,
  pipeline: Pipeline,
  response_cache: Arc<ResponseCache>,
  upstream_address: SocketAddr,
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
    let response_cache = Arc::new(ResponseCache::new(2048));

    let start = Instant::now();
    let rules = load_blocklists(config.blocklists, &cache_dir).await?;
    info!("loaded lists in {:.2?}", start.elapsed());

    let pipeline = Pipeline::new().add(Blocker::new(rules));

    Ok(Self {
      socket,
      pipeline,
      response_cache,
      upstream_address: config.upstream_address,
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
      MiddlewareResult::Next => self.forward(ctx, src).await?,
    }
    Ok(())
  }

  async fn forward(&self, ctx: &mut Context, src: SocketAddr) -> anyhow::Result<()> {
    let cache_key = ctx.cache_key();

    if let Some(ref key) = cache_key
      && let Some(cached) = self.response_cache.get_with_id(key, ctx.msg().id())
    {
      self.socket.send_to(&cached, src).await?;
      return Ok(());
    }

    let forward_bytes = ctx.msg().to_bytes()?;
    let socket = self.socket.clone();
    let response_cache = self.response_cache.clone();
    let upstream_addr = self.upstream_address;

    tokio::spawn(async move {
      let upstream = UdpSocket::bind("0.0.0.0:0").await?;
      upstream.send_to(&forward_bytes, upstream_addr).await?;

      let mut buf = vec![0u8; 512];
      let resp_len =
        match tokio::time::timeout(Duration::from_secs(5), upstream.recv_from(&mut buf))
          .await
        {
          Ok(Ok((len, _))) => len,
          Ok(Err(e)) => return Err(e.into()),
          Err(_) => return Ok(()),
        };

      let response = buf[..resp_len].to_vec();

      if let Some(key) = cache_key
        && let Ok(msg) = Message::from_bytes(&response)
        && msg.metadata.response_code == ResponseCode::NoError
        && let Some(ttl) = min_ttl(&msg)
      {
        response_cache.insert(key, response.clone(), Duration::from_secs(ttl as u64));
      }

      socket.send_to(&response, src).await?;
      Ok::<_, anyhow::Error>(())
    });

    Ok(())
  }
}

fn min_ttl(msg: &Message) -> Option<u32> {
  msg.answers.iter().map(|r| r.ttl).min().filter(|ttl| *ttl > 0)
}

pub async fn send_response(
  socket: &UdpSocket,
  src: SocketAddr,
  msg: Message,
) -> anyhow::Result<()> {
  socket.send_to(&msg.to_bytes()?, src).await?;
  Ok(())
}
