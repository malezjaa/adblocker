use crate::application::app::App;
use crate::context::Context;
use crate::engine::{BlockOrigin, process_message};
use anyhow::Result;
use hickory_proto::op::Message;
use hickory_proto::serialize::binary::BinDecodable;
use std::net::SocketAddr;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsStream};
use tracing::{debug, error, info};

impl App {
  pub async fn start_dot(ctx: Context) -> Result<()> {
    let acceptor = TlsAcceptor::from(ctx.server_config());

    let addr: SocketAddr = "0.0.0.0:853".parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!("DoT server listening on {addr}");

    loop {
      let (stream, peer) = listener.accept().await?;
      let acceptor = acceptor.clone();

      let ctx = ctx.clone();
      tokio::spawn(async move {
        debug!("connection from {peer}");
        match acceptor.accept(stream).await {
          Ok(tls_stream) => {
            if let Err(e) =
              Self::handle_connection(ctx, peer, TlsStream::from(tls_stream)).await
            {
              error!("Connection error: {e}");
            }
          }
          Err(e) => error!("TLS handshake failed: {e}"),
        }
      });
    }
  }

  async fn handle_connection(
    ctx: Context,
    peer: SocketAddr,
    mut stream: TlsStream<TcpStream>,
  ) -> Result<()> {
    loop {
      let mut len_buf = [0u8; 2];
      if stream.read_exact(&mut len_buf).await.is_err() {
        break;
      }
      let msg_len = u16::from_be_bytes(len_buf) as usize;

      let mut msg_buf = vec![0u8; msg_len];
      stream.read_exact(&mut msg_buf).await?;

      let msg = Message::from_bytes(&msg_buf)?;

      let start = Instant::now();
      let (blocked, response) =
        process_message(ctx.clone(), msg.to_vec()?, BlockOrigin::DoT).await?;
      ctx.db().spawn_query_record(
        &response,
        peer,
        blocked,
        BlockOrigin::DoT,
        start.elapsed().as_millis() as i64,
      );

      let response_bytes = response.to_vec()?;
      let len = (response_bytes.len() as u16).to_be_bytes();
      stream.write_all(&len).await?;
      stream.write_all(&response_bytes).await?;
    }
    Ok(())
  }
}
