use std::io::ErrorKind;

use tokio::net::UdpSocket;
use tracing::info;
use vox_dns::block_origin::BlockOrigin;

use crate::{application::app::App, context::Context};

impl App {
  pub async fn start_dns(ctx: Context) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 65_507];
    let socket = UdpSocket::bind(ctx.socket()).await?;

    info!("DNS server listening on {}", ctx.socket());

    loop {
      let (len, src) = match socket.recv_from(&mut buf).await {
        Ok(v) => v,
        Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
        Err(e) => return Err(e.into()),
      };
      let raw = buf[..len].to_vec();

      if raw.starts_with(b"health") {
        socket.send_to(b"ok", src).await?;
        continue;
      }

      let response = ctx.query_dns(raw, BlockOrigin::plain(), src, None).await?;
      socket.send_to(&response.maybe_truncate_for_udp()?, src).await?;
    }
  }
}
