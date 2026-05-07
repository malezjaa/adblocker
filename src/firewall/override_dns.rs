use anyhow::{Result, bail};
use std::net::SocketAddr;
use std::process::Command;
use tracing::info;

#[cfg(windows)]
pub fn override_default_dns(
  socket: SocketAddr,
  secondary: Option<SocketAddr>,
) -> Result<()> {
  use crate::windows::adapters::dns_servers_to_strings;
  use windows::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, NO_ERROR};
  use windows::Win32::NetworkManagement::IpHelper::{
    GAA_FLAG_INCLUDE_PREFIX, GetAdaptersAddresses, IF_TYPE_SOFTWARE_LOOPBACK,
    IF_TYPE_TUNNEL, IP_ADAPTER_ADDRESSES_LH,
  };
  use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;
  use windows::Win32::Networking::WinSock::AF_UNSPEC;

  let adapters = unsafe {
    let mut buf_len: u32 = 0;

    let ret = GetAdaptersAddresses(
      AF_UNSPEC.0 as u32,
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
      AF_UNSPEC.0 as u32,
      GAA_FLAG_INCLUDE_PREFIX,
      None,
      Some(adapter_ptr),
      &mut buf_len,
    );

    if ret != NO_ERROR.0 {
      bail!("GetAdaptersAddresses failed: {}", ret);
    }

    let mut current = adapter_ptr;

    let mut adapters: Vec<(String, Vec<String>)> = vec![];
    while !current.is_null() {
      let adapter = &*current;

      if adapter.OperStatus == IfOperStatusUp
        && adapter.IfType != IF_TYPE_SOFTWARE_LOOPBACK
        && adapter.IfType != IF_TYPE_TUNNEL
      {
        let name = adapter.FriendlyName.to_string()?;
        let existing_dns = dns_servers_to_strings(adapter);
        adapters.push((name, existing_dns));
      }

      current = adapter.Next;
    }

    adapters
  };

  let socket = socket.ip().to_string();
  let secondary_ip =
    secondary.map(|s| s.ip().to_string()).unwrap_or_else(|| "8.8.8.8".to_string());

  for (name, original) in adapters {
    info!("processing adapter: {}", name);

    let mut servers = vec![socket.clone()];
    servers.extend(original.iter().filter(|o| *o != &socket).cloned());

    if !servers.contains(&secondary_ip) {
      servers.push(secondary_ip.clone());
    }

    let servers_ps =
      servers.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(",");

    info!("servers_ps = {}", servers_ps);

    let script = format!(
      "Set-DnsClientServerAddress -InterfaceAlias '{name}' -ServerAddresses ({})",
      servers_ps
    );

    let status = Command::new("powershell")
      .args(["-NoProfile", "-NonInteractive", "-Command", &script])
      .status()?;

    if !status.success() {
      bail!("Set-DnsClientServerAddress failed");
    }
  }

  Ok(())
}
