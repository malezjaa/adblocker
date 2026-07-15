pub mod packet;

use std::{borrow::Cow, time::Instant};

use anyhow::Result;
use hickory_client::proto::op::Message;
use tracing::{debug, warn};
use vox_dns::{
  block_origin::{BlockOrigin, ClientOrigin, TransportOrigin},
  dns_query::DnsQuery,
  edns::EDNSCode,
};
use windivert::{
  WinDivert as Divert, address::WinDivertAddress, layer::NetworkLayer,
  packet::WinDivertPacket, prelude::WinDivertFlags,
};
use windivert_sys::ChecksumFlags;

use self::packet::{IpHeader, Packet, TransportHeader};
use crate::{config::WinClientConfig, upstream::UpstreamClient};

#[derive(Debug)]
pub struct WinDivert {
  pub divert: Divert<NetworkLayer>,
  pub config: WinClientConfig,
}

impl WinDivert {
  pub fn new(config: WinClientConfig) -> Result<Self> {
    let filter = format!(
      "outbound and (udp.DstPort == 53) and not loopback and ip.DstAddr != 127.0.0.1 \
       and ip.DstAddr != {}",
      config.dns_server.ip()
    );
    let divert = Divert::network(&filter, 0, WinDivertFlags::new())?;
    Ok(WinDivert { divert, config })
  }

  pub async fn start_redirects(self, upstream: UpstreamClient) -> Result<()> {
    let mut buf = vec![0u8; 65535];
    loop {
      let og_packet = self.divert.recv(&mut buf)?;
      let Some(packet) = Packet::parse(&og_packet.data) else { continue };

      if packet.payload.is_empty() || packet.payload.len() < 12 {
        continue;
      }
      if let Err(e) = self.handle_packet(&upstream, og_packet.address, packet).await {
        warn!("failed to process packet: {e:?}");
        continue;
      }
    }
  }

  async fn handle_packet(
    &self,
    upstream: &UpstreamClient,
    win_divert_address: WinDivertAddress<NetworkLayer>,
    packet: Packet<'_>,
  ) -> Result<()> {
    let start = Instant::now();
    let msg = Message::from_vec(packet.payload)?;
    let query_domain =
      msg.queries()[0].name().to_string().trim_end_matches('.').to_string();

    let origin = BlockOrigin::Client {
      transport: if self.config.using_doh() {
        TransportOrigin::DoH
      } else {
        TransportOrigin::Plain
      },
      client: ClientOrigin::Windows,
    };
    let response_bytes = upstream
      .send(
        DnsQuery::from_message(msg)
          .add_edns_option(EDNSCode::BlockOrigin, &[origin.to_u8()]),
      )
      .await?;

    debug!("dns request: {}ms src={query_domain}", start.elapsed().as_millis());

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
