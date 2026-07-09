use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::ptr;
use windivert_sys::WinDivertHelperParsePacket;
use windivert_sys::header::{
  PWINDIVERT_ICMPHDR, PWINDIVERT_ICMPV6HDR, PWINDIVERT_IPHDR, PWINDIVERT_IPV6HDR,
  PWINDIVERT_TCPHDR, PWINDIVERT_UDPHDR,
};

#[derive(Debug, Clone, Copy)]
pub enum IpHeader<'a> {
  V4(&'a [u8]),
  V6(&'a [u8]),
}

#[derive(Debug, Clone, Copy)]
pub enum TransportHeader<'a> {
  Udp(&'a [u8]),
  Tcp(&'a [u8]),
}

#[derive(Debug)]
pub struct Packet<'a> {
  pub ip_header: IpHeader<'a>,
  pub transport_header: Option<TransportHeader<'a>>,
  pub payload: &'a [u8],
  pub next: Option<&'a [u8]>,
}

impl<'a> Packet<'a> {
  pub fn is_ipv4(&self) -> bool {
    matches!(self.ip_header, IpHeader::V4(_))
  }

  pub fn is_ipv6(&self) -> bool {
    matches!(self.ip_header, IpHeader::V6(_))
  }

  pub fn source_addr(&self) -> SocketAddr {
    let ip = match self.ip_header {
      IpHeader::V4(hdr) if hdr.len() >= 16 => {
        IpAddr::V4(Ipv4Addr::new(hdr[12], hdr[13], hdr[14], hdr[15]))
      }
      IpHeader::V6(hdr) if hdr.len() >= 24 => {
        let mut octets = [0u8; 16];
        octets.copy_from_slice(&hdr[8..24]);
        IpAddr::V6(Ipv6Addr::from(octets))
      }
      IpHeader::V4(_) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
      IpHeader::V6(_) => IpAddr::V6(Ipv6Addr::UNSPECIFIED),
    };

    let port = match self.transport_header {
      Some(TransportHeader::Udp(hdr)) | Some(TransportHeader::Tcp(hdr))
        if hdr.len() >= 2 =>
      {
        u16::from_be_bytes([hdr[0], hdr[1]])
      }
      _ => 0,
    };

    SocketAddr::new(ip, port)
  }

  pub fn parse(data: &'a [u8]) -> Option<Packet<'a>> {
    let mut ip_hdr: PWINDIVERT_IPHDR = ptr::null_mut();
    let mut ipv6_hdr: PWINDIVERT_IPV6HDR = ptr::null_mut();
    let mut protocol: u8 = 0;
    let mut icmp_hdr: PWINDIVERT_ICMPHDR = ptr::null_mut();
    let mut icmpv6_hdr: PWINDIVERT_ICMPV6HDR = ptr::null_mut();
    let mut tcp_hdr: PWINDIVERT_TCPHDR = ptr::null_mut();
    let mut udp_hdr: PWINDIVERT_UDPHDR = ptr::null_mut();

    let mut data_ptr: *mut core::ffi::c_void = ptr::null_mut();
    let mut data_len: u32 = 0;
    let mut next_ptr: *mut core::ffi::c_void = ptr::null_mut();
    let mut next_len: u32 = 0;

    let ok = unsafe {
      WinDivertHelperParsePacket(
        data.as_ptr() as *const core::ffi::c_void,
        data.len() as u32,
        &mut ip_hdr,
        &mut ipv6_hdr,
        &mut protocol,
        &mut icmp_hdr,
        &mut icmpv6_hdr,
        &mut tcp_hdr,
        &mut udp_hdr,
        &mut data_ptr,
        &mut data_len,
        &mut next_ptr,
        &mut next_len,
      )
    };

    if ok == 0 {
      return None;
    }

    let buf_start = data.as_ptr() as usize;
    let buf_end = buf_start + data.len();

    let ip_start = if !ip_hdr.is_null() {
      ip_hdr as usize
    } else if !ipv6_hdr.is_null() {
      ipv6_hdr as usize
    } else {
      return None;
    };

    let transport_start =
      [tcp_hdr as usize, udp_hdr as usize, icmp_hdr as usize, icmpv6_hdr as usize]
        .into_iter()
        .find(|&p| p != 0);

    let payload_start = if !data_ptr.is_null() { data_ptr as usize } else { buf_end };

    let transport_end = payload_start;
    let ip_end = transport_start.unwrap_or(payload_start);

    if ip_start < buf_start || ip_end < ip_start || ip_end > buf_end {
      return None;
    }

    let ip_header = unsafe {
      let slice = std::slice::from_raw_parts(ip_start as *const u8, ip_end - ip_start);
      if !ip_hdr.is_null() { IpHeader::V4(slice) } else { IpHeader::V6(slice) }
    };

    let transport_header = transport_start.map(|t_start| unsafe {
      let len = transport_end.saturating_sub(t_start);
      let slice = std::slice::from_raw_parts(t_start as *const u8, len);
      if !tcp_hdr.is_null() {
        TransportHeader::Tcp(slice)
      } else if !udp_hdr.is_null() {
        TransportHeader::Udp(slice)
      } else {
        unreachable!()
      }
    });

    let payload = if !data_ptr.is_null() && data_len > 0 {
      unsafe { std::slice::from_raw_parts(data_ptr as *const u8, data_len as usize) }
    } else {
      &[]
    };

    let next = if !next_ptr.is_null() && next_len > 0 {
      Some(unsafe {
        std::slice::from_raw_parts(next_ptr as *const u8, next_len as usize)
      })
    } else {
      None
    };

    Some(Packet { ip_header, transport_header, payload, next })
  }

  pub fn port_offsets(&self, data: &'a [u8]) -> Option<(usize, usize)> {
    let hdr = match self.transport_header {
      Some(TransportHeader::Tcp(hdr)) | Some(TransportHeader::Udp(hdr)) => hdr,
      _ => return None,
    };

    if hdr.len() < 4 {
      return None;
    }

    let buf_start = data.as_ptr() as usize;
    let buf_end = buf_start + data.len();
    let hdr_start = hdr.as_ptr() as usize;

    if hdr_start < buf_start || hdr_start + hdr.len() > buf_end {
      return None;
    }

    let src_offset = hdr_start - buf_start;
    let dst_offset = src_offset + 2;

    Some((src_offset, dst_offset))
  }

  pub fn src_port_offset(&self, data: &'a [u8]) -> Option<usize> {
    self.port_offsets(data).map(|(s, _)| s)
  }

  pub fn dst_port_offset(&self, data: &'a [u8]) -> Option<usize> {
    self.port_offsets(data).map(|(_, d)| d)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn source_addr_reads_ipv4_source_and_transport_port() {
    let mut ip = [0u8; 20];
    ip[12..16].copy_from_slice(&[192, 0, 2, 10]);
    let transport = [0x1f, 0x90, 0x00, 0x35];
    let packet = Packet {
      ip_header: IpHeader::V4(&ip),
      transport_header: Some(TransportHeader::Udp(&transport)),
      payload: &[],
      next: None,
    };

    assert!(packet.is_ipv4());
    assert!(!packet.is_ipv6());
    assert_eq!(
      packet.source_addr(),
      SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 10)), 8080)
    );
  }

  #[test]
  fn source_addr_reads_ipv6_source_and_tcp_port() {
    let mut ip = [0u8; 40];
    let source = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1).octets();
    ip[8..24].copy_from_slice(&source);
    let transport = [0x01, 0xbb, 0x00, 0x35];
    let packet = Packet {
      ip_header: IpHeader::V6(&ip),
      transport_header: Some(TransportHeader::Tcp(&transport)),
      payload: &[],
      next: None,
    };

    assert!(!packet.is_ipv4());
    assert!(packet.is_ipv6());
    assert_eq!(
      packet.source_addr(),
      SocketAddr::new(IpAddr::V6(Ipv6Addr::from(source)), 443)
    );
  }

  #[test]
  fn source_addr_uses_unspecified_ip_and_zero_port_for_short_headers() {
    let ip = [0u8; 8];
    let transport = [0x1f];
    let packet = Packet {
      ip_header: IpHeader::V4(&ip),
      transport_header: Some(TransportHeader::Udp(&transport)),
      payload: &[],
      next: None,
    };

    assert_eq!(
      packet.source_addr(),
      SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)
    );
  }

  #[test]
  fn port_offsets_return_source_and_destination_positions_inside_packet() {
    let data = [0u8; 32];
    let transport = &data[20..28];
    let packet = Packet {
      ip_header: IpHeader::V4(&data[..20]),
      transport_header: Some(TransportHeader::Udp(transport)),
      payload: &[],
      next: None,
    };

    assert_eq!(packet.port_offsets(&data), Some((20, 22)));
    assert_eq!(packet.src_port_offset(&data), Some(20));
    assert_eq!(packet.dst_port_offset(&data), Some(22));
  }

  #[test]
  fn port_offsets_reject_transport_header_outside_packet_buffer() {
    let data = [0u8; 32];
    let packet = Packet {
      ip_header: IpHeader::V4(&data[..20]),
      transport_header: Some(TransportHeader::Udp(&data[20..28])),
      payload: &[],
      next: None,
    };

    assert_eq!(packet.port_offsets(&data[..20]), None);
  }

  #[test]
  fn port_offsets_require_at_least_four_transport_bytes() {
    let data = [0u8; 24];
    let packet = Packet {
      ip_header: IpHeader::V4(&data[..20]),
      transport_header: Some(TransportHeader::Udp(&data[20..23])),
      payload: &[],
      next: None,
    };

    assert_eq!(packet.port_offsets(&data), None);
  }
}
