use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum ClientOrigin {
  Windows = 0,
  Linux = 1,
  Mac = 2,
}

impl ClientOrigin {
  fn from_u8(v: u8) -> anyhow::Result<Self> {
    Ok(match v {
      0 => Self::Windows,
      1 => Self::Linux,
      2 => Self::Mac,
      _ => anyhow::bail!("invalid ClientOrigin"),
    })
  }
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum TransportOrigin {
  Plain = 0,
  DoH = 1,
  DoT = 2,
  DoQ = 3,
}

impl TransportOrigin {
  fn from_u8(v: u8) -> anyhow::Result<Self> {
    Ok(match v {
      0 => Self::Plain,
      1 => Self::DoH,
      2 => Self::DoT,
      3 => Self::DoQ,
      _ => anyhow::bail!("invalid TransportOrigin"),
    })
  }
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum BlockOrigin {
  Transport(TransportOrigin),
  Client { client: ClientOrigin, transport: TransportOrigin },
}

impl BlockOrigin {
  const TRANSPORT_MASK: u8 = 0b0000_0011;
  const CLIENT_MASK: u8 = 0b0000_1100;
  const CLIENT_SHIFT: u8 = 2;

  pub fn to_u8(self) -> u8 {
    match self {
      BlockOrigin::Transport(t) => Self::CLIENT_MASK | t as u8,
      BlockOrigin::Client { client, transport } => {
        let c = (client as u8) << Self::CLIENT_SHIFT;
        let t = transport as u8;
        c | t
      }
    }
  }

  pub fn from_u8(v: u8) -> anyhow::Result<Self> {
    let transport = TransportOrigin::from_u8(v & Self::TRANSPORT_MASK)?;
    let client_raw = (v & Self::CLIENT_MASK) >> Self::CLIENT_SHIFT;

    Ok(match client_raw {
      0b11 => BlockOrigin::Transport(transport),
      0..=2 => {
        BlockOrigin::Client { client: ClientOrigin::from_u8(client_raw)?, transport }
      }
      _ => anyhow::bail!("invalid BlockOrigin"),
    })
  }

  pub fn plain() -> Self {
    Self::Transport(TransportOrigin::Plain)
  }

  pub fn doh() -> Self {
    Self::Transport(TransportOrigin::DoH)
  }

  pub fn dot() -> Self {
    Self::Transport(TransportOrigin::DoT)
  }

  pub fn doq() -> Self {
    Self::Transport(TransportOrigin::DoQ)
  }
}

impl From<BlockOrigin> for u8 {
  fn from(v: BlockOrigin) -> Self {
    v.to_u8()
  }
}

impl TryFrom<u8> for BlockOrigin {
  type Error = anyhow::Error;

  fn try_from(v: u8) -> anyhow::Result<Self> {
    Self::from_u8(v)
  }
}

impl fmt::Debug for BlockOrigin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      BlockOrigin::Transport(t) => {
        write!(f, "{t:?}")
      }
      BlockOrigin::Client { client, transport } => {
        write!(f, "C({:?},{:?})", client, transport)
      }
    }
  }
}

impl fmt::Debug for ClientOrigin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match self {
      ClientOrigin::Windows => "Win",
      ClientOrigin::Linux => "Lin",
      ClientOrigin::Mac => "Mac",
    };
    write!(f, "{s}")
  }
}

impl fmt::Debug for TransportOrigin {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match self {
      TransportOrigin::Plain => "Plain",
      TransportOrigin::DoH => "DoH",
      TransportOrigin::DoT => "DoT",
      TransportOrigin::DoQ => "DoQ",
    };
    write!(f, "{s}")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn transport_origins_round_trip_through_byte_encoding() {
    for origin in
      [BlockOrigin::plain(), BlockOrigin::doh(), BlockOrigin::dot(), BlockOrigin::doq()]
    {
      assert_eq!(BlockOrigin::from_u8(origin.to_u8()).unwrap(), origin);
    }
  }

  #[test]
  fn client_origins_round_trip_through_byte_encoding() {
    for client in [ClientOrigin::Windows, ClientOrigin::Linux, ClientOrigin::Mac] {
      for transport in [
        TransportOrigin::Plain,
        TransportOrigin::DoH,
        TransportOrigin::DoT,
        TransportOrigin::DoQ,
      ] {
        let origin = BlockOrigin::Client { client, transport };

        assert_eq!(BlockOrigin::from_u8(origin.to_u8()).unwrap(), origin);
      }
    }
  }

  #[test]
  fn byte_encoding_keeps_transport_only_values_out_of_client_range() {
    assert_eq!(BlockOrigin::plain().to_u8(), 0b0000_1100);
    assert_eq!(BlockOrigin::doh().to_u8(), 0b0000_1101);
    assert_eq!(BlockOrigin::dot().to_u8(), 0b0000_1110);
    assert_eq!(BlockOrigin::doq().to_u8(), 0b0000_1111);
  }
}
