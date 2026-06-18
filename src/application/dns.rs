use crate::application::app::App;
use crate::context::Context;
use crate::dns::process::process_message;
use crate::engine::message::BlockOrigin;
use std::io::ErrorKind;
use tokio::net::UdpSocket;
use tokio::time::Instant;
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

      let start = Instant::now();
      let (blocked, response) =
        process_message(ctx.clone(), raw, BlockOrigin::Plain).await?;
      let elapsed = start.elapsed();

      socket.send_to(&response.to_vec()?, src).await?;
      ctx.db().record_query(
        &response,
        src,
        blocked,
        BlockOrigin::Plain,
        elapsed.as_millis() as i64,
        None,
      );
    }
  }
}
