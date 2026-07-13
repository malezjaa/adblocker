use std::{io::ErrorKind, net::SocketAddr, time::Duration};

use anyhow::bail;
use tokio::{net::UdpSocket, time::timeout};

pub async fn check_server_health(addr: &SocketAddr) -> anyhow::Result<()> {
  let socket = UdpSocket::bind("0.0.0.0:0").await?;
  socket.connect(addr).await?;
  socket.send(b"health").await?;

  let mut buf = [0u8; 64];
  let len = match timeout(Duration::from_secs(1), socket.recv(&mut buf)).await {
    Ok(Ok(len)) => len,
    Ok(Err(err)) => match err.kind() {
      ErrorKind::ConnectionReset
      | ErrorKind::ConnectionRefused
      | ErrorKind::ConnectionAborted => {
        bail!("DNS server is not running");
      }
      _ => return Err(err.into()),
    },
    Err(_) => bail!("Timed out waiting for DNS server"),
  };

  let response = &buf[..len];
  if response != b"ok" {
    bail!("You specified a DNS server that isn't vox. Some features won't work.");
  }

  Ok(())
}
