mod middleware;
mod context;
mod config;
mod blocklists;
mod middlewares;
mod pipeline;
mod cache;

use crate::blocklists::load_blocklists;
use crate::config::Config;
use crate::context::Context;
use crate::middleware::MiddlewareResult;
use crate::pipeline::Pipeline;
use anyhow::Result;
use fs_err::create_dir_all;
use hickory_proto::op::{Message, UpdateMessage};
use hickory_proto::serialize::binary::{BinDecodable, BinEncodable};
use middlewares::blocker::Blocker;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tracing::{debug, error, info};

fn setup_logger() {
  tracing_subscriber::fmt()
    .with_env_filter("dns_adblock=info")
    .init();
}

#[tokio::main]
async fn main() -> Result<()> {
  setup_logger();

  loop {
    match run().await {
      Ok(_) => {}
      Err(err) => {
        error!(error = ?err, "dns adblocker failed. trying to restart")
      }
    }
  }
}

async fn run() -> Result<()> {
  let home_path = dirs::home_dir()
    .unwrap()
    .join("adb");
  let config = Config::from_file(home_path.join("config.toml"))?;
  let cache_dir = home_path.join("cache");

  if !cache_dir.exists() {
    create_dir_all(&cache_dir)?;
  }

  let socket = Arc::new(UdpSocket::bind(config.socket).await?);
  let upstream = UdpSocket::bind("0.0.0.0:0").await?;

  let start = Instant::now();
  let rules = load_blocklists(config.blocklists, &cache_dir).await?;
  info!("loaded lists in {:.2?}", start.elapsed());

  let pipeline = Pipeline::new()
    .add(Blocker::new(rules));

  let mut buf = vec![0u8; 512];
  loop {
    let (len, src) = match socket.recv_from(&mut buf).await {
      Ok(v) => v,

      Err(e) if e.kind() == ErrorKind::ConnectionReset => {
        continue;
      }

      Err(e) => return Err(e.into()),
    };

    let raw = buf[..len].to_vec();
    let mut ctx = Context::new(raw)?;
    debug!(request = ?ctx, from = %src);

    match pipeline.run(&mut ctx).await {
      MiddlewareResult::Block => {
        ctx.send_blocked(&socket, src).await?;
      }

      MiddlewareResult::Respond(msg) => {
        send_response(&socket, src, msg).await?;
      }

      MiddlewareResult::Next => {
        let forward_bytes = ctx.msg().to_bytes()?;
        let socket = socket.clone();

        tokio::spawn(async move {
          let upstream = UdpSocket::bind("0.0.0.0:0").await?;
          upstream.send_to(&forward_bytes, config.upstream_address).await?;

          let mut buf = vec![0u8; 512];
          let resp_len = match tokio::time::timeout(
            Duration::from_secs(5),
            upstream.recv_from(&mut buf),
          ).await {
            Ok(Ok((len, _))) => len,
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => return Ok(()),
          };

          let response = buf[..resp_len].to_vec();

          socket.send_to(&response, src).await?;
          Ok::<_, anyhow::Error>(())
        });
      }
    }
  }
}

pub async fn send_response(
  socket: &UdpSocket,
  src: SocketAddr,
  msg: Message,
) -> Result<()> {
  let bytes = msg.to_bytes()?;
  socket.send_to(&bytes, src).await?;
  Ok(())
}
