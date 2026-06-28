use crate::application::app::App;
use crate::context::Context;
use crate::engine::message::BlockOrigin;
use std::io::ErrorKind;
use tokio::net::UdpSocket;
use tracing::info;

impl App {
  pub async fn start_dns(ctx: Context) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 512];
    let socket = UdpSocket::bind(Context::socket()).await?;

    info!("DNS server listening on {}", Context::socket());

    loop {
      let (len, src) = match socket.recv_from(&mut buf).await {
        Ok(v) => v,
        Err(e) if e.kind() == ErrorKind::ConnectionReset => continue,
        Err(e) => return Err(e.into()),
      };

      let raw = buf[..len].to_vec();

      let response = ctx.query_dns(raw, BlockOrigin::Plain, src, None).await?;
      socket.send_to(&response.to_vec()?, src).await?;
    }
  }
}
