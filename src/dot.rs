use crate::blocker::{check_block, BlockOrigin};
use crate::cert::Certs;
use crate::state::State;
use anyhow::Result;
use hickory_proto::op::Message;
use hickory_proto::serialize::binary::BinDecodable;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsStream};
use tracing::{debug, error, info};

pub async fn setup_dot_server<'a>(state: State, certs: Certs) -> Result<()> {
  let config = ServerConfig::builder().with_no_client_auth().with_single_cert(certs.certs, certs.key)?;
  let acceptor = TlsAcceptor::from(Arc::new(config));

  let addr: SocketAddr = "0.0.0.0:853".parse()?;
  let listener = TcpListener::bind(addr).await?;
  info!("DoT server listening on {addr}");

  loop {
    let (stream, peer) = listener.accept().await?;
    let acceptor = acceptor.clone();

    let s = state.clone();
    tokio::spawn(async move {
      debug!("connection from {peer}");
      match acceptor.accept(stream).await {
        Ok(tls_stream) => {
          if let Err(e) = handle_connection(s, peer, TlsStream::from(tls_stream)).await {
            error!("Connection error: {e}");
          }
        }
        Err(e) => error!("TLS handshake failed: {e}"),
      }
    });
  }

  Ok(())
}

async fn handle_connection(
  state: State,
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
    let (blocked, response) = check_block(state.clone(), msg.to_vec()?, BlockOrigin::DoT).await?;
    state.spawn_query_record(&response, peer, blocked, BlockOrigin::DoT, start.elapsed().as_millis() as i64);

    let response_bytes = response.to_vec()?;
    let len = (response_bytes.len() as u16).to_be_bytes();
    stream.write_all(&len).await?;
    stream.write_all(&response_bytes).await?;
  }
  Ok(())
}