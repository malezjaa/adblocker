use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use windows::Win32::{
  NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH,
  Networking::WinSock::{AF_INET, AF_INET6, SOCKADDR_IN, SOCKADDR_IN6},
};

pub unsafe fn dns_servers_to_strings(adapter: &IP_ADAPTER_ADDRESSES_LH) -> Vec<String> {
  unsafe {
    let mut result = vec![];
    let mut current = adapter.FirstDnsServerAddress;

    while !current.is_null() {
      let dns = &*current;
      let sockaddr = dns.Address.lpSockaddr;

      if !sockaddr.is_null() {
        match (*sockaddr).sa_family {
          AF_INET => {
            let addr = *(sockaddr as *const SOCKADDR_IN);

            let bytes = addr.sin_addr.S_un.S_un_b;
            let ip = Ipv4Addr::new(bytes.s_b1, bytes.s_b2, bytes.s_b3, bytes.s_b4);

            result.push(IpAddr::V4(ip).to_string());
          }

          AF_INET6 => {
            let addr = *(sockaddr as *const SOCKADDR_IN6);

            let bytes = addr.sin6_addr.u.Byte;
            let ip = Ipv6Addr::from(bytes);

            result.push(IpAddr::V6(ip).to_string());
          }

          _ => {}
        }
      }

      current = dns.Next;
    }

    result
  }
}
