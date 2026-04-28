mod middleware;
mod context;
mod config;
mod blocklists;
mod middlewares;
mod pipeline;

use crate::blocklists::load_blocklists;
use crate::config::Config;
use crate::context::Context;
use crate::middleware::MiddlewareResult;
use crate::pipeline::Pipeline;
use anyhow::Result;
use hickory_proto::op::Message;
use hickory_proto::serialize::binary::BinEncodable;
use middlewares::blocker::Blocker;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::time::Instant;
use tokio::net::UdpSocket;
use tracing::{debug, error, info};

fn setup_logger() {
  tracing_subscriber::fmt()
    .with_env_filter("dns_adblock=debug")
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
  let config_path = dirs::home_dir()
    .unwrap()
    .join("adb")
    .join("config.toml");

  let config = Config::from_file(config_path)?;

  let socket = UdpSocket::bind(config.socket).await?;
  let upstream = UdpSocket::bind("0.0.0.0:0").await?;

  let start = Instant::now();
  let rules = load_blocklists(config.blocklists).await?;
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

    let raw: Vec<u8> = buf[..len].to_vec();
    let mut ctx = Context::new(&raw)?;

    debug!(request = ?ctx, from = %src);

    match pipeline.run(&mut ctx).await {
      MiddlewareResult::Block => {
        ctx.send_blocked(&socket, src).await?;
      }

      MiddlewareResult::Respond(msg) => {
        send_response(&socket, src, msg).await?;
      }

      MiddlewareResult::Next => {
        let forward_bytes = ctx.msg().to_bytes().unwrap_or(raw);
        upstream
          .send_to(&forward_bytes, config.upstream_address)
          .await?;

        let (resp_len, _) = match upstream.recv_from(&mut buf).await {
          Ok(v) => v,
          Err(e) if e.kind() == ErrorKind::ConnectionReset => {
            continue;
          }
          Err(e) => return Err(e.into()),
        };

        socket
          .send_to(&buf[..resp_len], src)
          .await?;
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
