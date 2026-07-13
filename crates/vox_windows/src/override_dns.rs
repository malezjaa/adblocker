use std::net::SocketAddr;

use anyhow::{Result, bail};
use windows::{
  Win32::NetworkManagement::{
    IpHelper::{
      ConvertInterfaceLuidToGuid, DNS_DOH_SERVER_SETTINGS,
      DNS_DOH_SERVER_SETTINGS_ENABLE, DNS_INTERFACE_SETTINGS,
      DNS_INTERFACE_SETTINGS_VERSION3, DNS_INTERFACE_SETTINGS3, DNS_SERVER_PROPERTY,
      DNS_SERVER_PROPERTY_TYPE, DNS_SERVER_PROPERTY_TYPES, DNS_SETTING_DOH,
      DNS_SETTING_NAMESERVER, SetInterfaceDnsSettings,
    },
    Ndis::NET_LUID_LH,
  },
  core::GUID,
};

use crate::pwstr_buf::PwstrBuffer;

#[derive(Debug)]
pub struct OverrideDns {
  pub socket: SocketAddr,
  pub secondary: Option<SocketAddr>,
  pub doh: Option<String>,
}

#[allow(unused_assignments)]
pub fn override_default_dns(settings: OverrideDns) -> Result<()> {
  use windows::Win32::{
    Foundation::{ERROR_BUFFER_OVERFLOW, NO_ERROR},
    NetworkManagement::{
      IpHelper::{
        GAA_FLAG_INCLUDE_PREFIX, GetAdaptersAddresses, IF_TYPE_SOFTWARE_LOOPBACK,
        IF_TYPE_TUNNEL, IP_ADAPTER_ADDRESSES_LH,
      },
      Ndis::IfOperStatusUp,
    },
    Networking::WinSock::AF_UNSPEC,
  };

  use crate::adapters::dns_servers_to_strings;
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

    let mut adapters: Vec<(NET_LUID_LH, Vec<String>)> = vec![];
    while !current.is_null() {
      let adapter = &*current;

      if adapter.OperStatus == IfOperStatusUp
        && adapter.IfType != IF_TYPE_SOFTWARE_LOOPBACK
        && adapter.IfType != IF_TYPE_TUNNEL
      {
        let _name = adapter.FriendlyName.to_string()?;
        let existing_dns = dns_servers_to_strings(adapter);
        adapters.push((adapter.Luid, existing_dns));
      }

      current = adapter.Next;
    }

    adapters
  };

  let primary = settings.socket.ip().to_string();
  let secondary = settings
    .secondary
    .map(|s| s.ip().to_string())
    .unwrap_or_else(|| "1.1.1.1".to_string());

  for (luid, name_servers) in adapters {
    let mut servers = vec![primary.clone()];
    servers.extend(name_servers.into_iter().filter(|s| s != &primary));

    if !servers.contains(&secondary) {
      servers.push(secondary.clone());
    }

    let dns_string = servers.join(",");
    let ns = PwstrBuffer::new(&dns_string);

    let doh_template;
    let mut doh_settings;
    let server_property;

    let (flags, c_server_props, server_props_ptr) = if let Some(doh) = &settings.doh {
      doh_template = PwstrBuffer::new(doh);
      doh_settings = DNS_DOH_SERVER_SETTINGS {
        Template: doh_template.as_pwstr(),
        Flags: DNS_DOH_SERVER_SETTINGS_ENABLE as u64,
      };
      server_property = DNS_SERVER_PROPERTY {
        Version: 1,
        ServerIndex: 0,
        Type: DNS_SERVER_PROPERTY_TYPE(1),
        Property: DNS_SERVER_PROPERTY_TYPES { DohSettings: &mut doh_settings },
      };
      (
        (DNS_SETTING_NAMESERVER | DNS_SETTING_DOH) as u64,
        1u32,
        &server_property as *const _ as *mut DNS_SERVER_PROPERTY,
      )
    } else {
      doh_template = PwstrBuffer::new("");
      doh_settings = DNS_DOH_SERVER_SETTINGS::default();
      server_property = DNS_SERVER_PROPERTY::default();
      (DNS_SETTING_NAMESERVER as u64, 0u32, std::ptr::null_mut())
    };

    let settings = DNS_INTERFACE_SETTINGS3 {
      Version: DNS_INTERFACE_SETTINGS_VERSION3,
      Flags: flags,
      NameServer: ns.as_pwstr(),
      cServerProperties: c_server_props,
      ServerProperties: server_props_ptr,
      ..Default::default()
    };

    unsafe {
      let mut guid = GUID::default();
      let status = ConvertInterfaceLuidToGuid(&luid, &mut guid);
      if status.0 != 0 {
        bail!("ConvertInterfaceLuidToGuild failed: {status:?}")
      }

      let status = SetInterfaceDnsSettings(
        guid,
        &settings as *const _ as *const DNS_INTERFACE_SETTINGS,
      );
      if status.0 != 0 {
        bail!("SetInterfaceDnsSettings failed: {status:?}");
      }
    }
  }

  Ok(())
}
