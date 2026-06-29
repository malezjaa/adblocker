pub mod packet;

use crate::context::Context;
use crate::engine::message::BlockOrigin;
use crate::win_divert::packet::{IpHeader, Packet, TransportHeader};
use anyhow::Result;
use std::borrow::Cow;
use tracing::warn;
use windivert::WinDivert as Divert;
use windivert::address::WinDivertAddress;
use windivert::layer::NetworkLayer;
use windivert::packet::WinDivertPacket;
use windivert::prelude::WinDivertFlags;
use windivert_sys::ChecksumFlags;

#[derive(Debug)]
pub struct WinDivert {
  pub divert: Divert<NetworkLayer>,
}

impl WinDivert {
  pub fn new() -> Result<Self> {
    let divert = Divert::network(
      "outbound and (udp.DstPort == 53 or tcp.DstPort == 53) and ip.DstAddr != 127.0.0.1",
      0,
      WinDivertFlags::new(),
    )?;
    Ok(WinDivert { divert })
  }

  pub async fn start_redirects(&self, ctx: Context) -> Result<()> {
    let mut buf = vec![0u8; 65535];
    loop {
      let og_packet = self.divert.recv(&mut buf)?;
      let Some(packet) = Packet::parse(&og_packet.data) else { continue };

      if packet.payload.is_empty() || packet.payload.len() < 12 {
        continue;
      }
      if let Err(e) = self.handle_packet(ctx.clone(), og_packet.address, packet).await {
        warn!("failed to process packet: {e:?}");
        continue;
      }
    }
  }

  async fn handle_packet(
    &self,
    ctx: Context,
    win_divert_address: WinDivertAddress<NetworkLayer>,
    packet: Packet<'_>,
  ) -> Result<()> {
    let response = ctx
      .query_dns(
        packet.payload.to_owned(),
        BlockOrigin::PlainWinDivert,
        packet.source_addr(),
        None,
      )
      .await?;
    let mut ip_header = match packet.ip_header.to_owned() {
      IpHeader::V4(ip_header) => {
        let mut ip_header = ip_header.to_owned();
        let dst = ip_header[16..20].to_vec();
        let src = ip_header[12..16].to_vec();

        ip_header[12..16].copy_from_slice(&dst);
        ip_header[16..20].copy_from_slice(&src);
        ip_header
      }
      IpHeader::V6(ip_header) => {
        let mut ip_header = ip_header.to_owned();
        let src = ip_header[24..40].to_vec();
        let dst = ip_header[8..24].to_vec();

        ip_header[8..24].copy_from_slice(&src);
        ip_header[24..40].copy_from_slice(&dst);
        ip_header
      }
    };

    let mut transport_header = match packet.transport_header {
      Some(TransportHeader::Tcp(transport)) | Some(TransportHeader::Udp(transport)) => {
        let mut transport = transport.to_owned();

        let mut src = [0u8; 2];
        let mut dst = [0u8; 2];
        src.copy_from_slice(&transport[0..2]);
        dst.copy_from_slice(&transport[2..4]);
        transport[0..2].copy_from_slice(&dst);
        transport[2..4].copy_from_slice(&src);

        transport
      }
      None => vec![],
    };

    let response_bytes = response.to_vec()?;

    let total_payload_len = transport_header.len() + response_bytes.len();

    // update length fields
    if let Some(TransportHeader::Udp(_)) = packet.transport_header {
      transport_header[4..6].copy_from_slice(&(total_payload_len as u16).to_be_bytes());
    }

    match packet.ip_header {
      IpHeader::V4(_) => {
        let total_len = (ip_header.len() + total_payload_len) as u16;
        ip_header[2..4].copy_from_slice(&total_len.to_be_bytes());
      }
      IpHeader::V6(_) => {
        ip_header[4..6].copy_from_slice(&(total_payload_len as u16).to_be_bytes());
      }
    }

    let mut packet_data = vec![];
    packet_data.extend(ip_header);
    packet_data.extend(transport_header);
    packet_data.extend(response_bytes);

    let mut packet = WinDivertPacket {
      address: win_divert_address,
      data: Cow::Owned(packet_data.clone()),
    };
    packet.address.set_outbound(false);
    packet.recalculate_checksums(ChecksumFlags::new())?;

    self.divert.send(&packet)?;
    Ok(())
  }
}
