use std::{
  net::{IpAddr, Ipv4Addr, Ipv6Addr},
  str::FromStr,
};

use anyhow::{Result, bail};
use windows::Win32::{
  Foundation::{ERROR_BUFFER_OVERFLOW, NO_ERROR},
  NetworkManagement::{
    IpHelper::{
      GAA_FLAG_INCLUDE_PREFIX, GetAdaptersAddresses, GetBestInterfaceEx,
      IF_TYPE_SOFTWARE_LOOPBACK, IF_TYPE_TUNNEL, IP_ADAPTER_ADDRESSES_LH,
    },
    Ndis::IfOperStatusUp,
  },
  Networking::WinSock::{AF_INET, AF_INET6, SOCKADDR, SOCKADDR_IN, SOCKADDR_IN6},
};

#[derive(Debug)]
pub struct AdapterInfo {
  pub name: String,
  pub index: u32,
  pub ipv4: Vec<IpAddr>,
  pub ipv6: Vec<IpAddr>,
}

impl AdapterInfo {
  pub fn best_interface_for_ipv4(target: &str) -> u32 {
    unsafe {
      let mut addr: SOCKADDR_IN = std::mem::zeroed();
      addr.sin_family = AF_INET;

      let ip = Ipv4Addr::from_str(target).unwrap();
      addr.sin_addr.S_un.S_addr = u32::from_be_bytes(ip.octets()).to_be();

      let mut index: u32 = 0;

      let res = GetBestInterfaceEx(&addr as *const _ as *const SOCKADDR, &mut index);

      if res == 0 { index } else { 0 }
    }
  }

  pub fn pick_ipv4(&self) -> Result<Ipv4Addr> {
    self
      .ipv4
      .iter()
      .filter_map(|ip| match ip {
        IpAddr::V4(v4) => Some(*v4),
        IpAddr::V6(_) => None,
      })
      .find(|v4| !v4.is_link_local())
      .or_else(|| {
        self.ipv4.iter().find_map(|ip| match ip {
          IpAddr::V4(v4) => Some(*v4),
          IpAddr::V6(_) => None,
        })
      })
      .ok_or_else(|| anyhow::anyhow!("Adapter {} has no IPv4 address", self.name))
  }
}

unsafe fn sockaddr_to_ip(sa: *const SOCKADDR) -> Option<IpAddr> {
  unsafe {
    if sa.is_null() {
      return None;
    }

    let family = (*sa).sa_family;

    match family {
      AF_INET => {
        let addr = &*(sa as *const SOCKADDR_IN);
        let ip = u32::from_be(addr.sin_addr.S_un.S_addr);
        Some(IpAddr::V4(ip.into()))
      }
      AF_INET6 => {
        let addr = &*(sa as *const SOCKADDR_IN6);
        Some(IpAddr::V6(Ipv6Addr::from(addr.sin6_addr.u.Byte)))
      }
      _ => None,
    }
  }
}

unsafe fn wide_ptr_to_string(ptr: *const u16) -> String {
  unsafe {
    if ptr.is_null() {
      return String::new();
    }

    let mut len = 0;
    while *ptr.add(len) != 0 {
      len += 1;
    }

    let slice = std::slice::from_raw_parts(ptr, len);
    String::from_utf16_lossy(slice)
  }
}

pub fn get_active_adapters() -> Result<Vec<AdapterInfo>> {
  unsafe {
    let mut buf_len: u32 = 0;

    let ret = GetAdaptersAddresses(
      windows::Win32::Networking::WinSock::AF_UNSPEC.0 as u32,
      GAA_FLAG_INCLUDE_PREFIX,
      None,
      None,
      &mut buf_len,
    );

    if ret != ERROR_BUFFER_OVERFLOW.0 {
      bail!("Failed to get buffer size: {}", ret);
    }

    let mut buffer = vec![0u8; buf_len as usize];
    let adapter_ptr = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

    let ret = GetAdaptersAddresses(
      windows::Win32::Networking::WinSock::AF_UNSPEC.0 as u32,
      GAA_FLAG_INCLUDE_PREFIX,
      None,
      Some(adapter_ptr),
      &mut buf_len,
    );

    if ret != NO_ERROR.0 {
      bail!("GetAdaptersAddresses failed: {}", ret);
    }

    let mut result = vec![];
    let mut current = adapter_ptr;

    while !current.is_null() {
      let adapter = &*current;

      if adapter.OperStatus == IfOperStatusUp
        && adapter.IfType != IF_TYPE_SOFTWARE_LOOPBACK
        && adapter.IfType != IF_TYPE_TUNNEL
      {
        let mut ipv4 = vec![];
        let mut ipv6 = vec![];

        let mut ua = adapter.FirstUnicastAddress;

        while !ua.is_null() {
          let ua_ref = &*ua;

          if let Some(ip) = sockaddr_to_ip(ua_ref.Address.lpSockaddr as *const SOCKADDR) {
            match ip {
              IpAddr::V4(_) => ipv4.push(ip),
              IpAddr::V6(_) => ipv6.push(ip),
            }
          }

          ua = ua_ref.Next;
        }

        if !ipv4.is_empty() || !ipv6.is_empty() {
          let name = wide_ptr_to_string(adapter.FriendlyName.0);
          result.push(AdapterInfo {
            name,
            index: adapter.Anonymous1.Anonymous.IfIndex,
            ipv4,
            ipv6,
          });
        }
      }

      current = adapter.Next;
    }

    Ok(result)
  }
}

pub fn primary_adapter() -> Result<Option<AdapterInfo>> {
  let adapters = get_active_adapters()?;
  let idx = AdapterInfo::best_interface_for_ipv4("1.1.1.1");

  if idx == 0 {
    return Ok(None);
  }

  Ok(adapters.into_iter().find(|a| a.index == idx))
}
