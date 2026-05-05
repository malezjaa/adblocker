use crate::fwpm_transaction;
use crate::windows::filter::{add, condition_remote_addr_v4, condition_remote_port, FilterBuilder};
use anyhow::{bail, Result};
use std::net::{IpAddr, SocketAddr};
use std::ptr::null;

#[cfg(windows)]
pub fn block_external_dns(resolver: SocketAddr) -> Result<()> {
  use windows::{
    core::PWSTR,
    Win32::Foundation::HANDLE,
    Win32::NetworkManagement::WindowsFilteringPlatform::*,
    Win32::System::Rpc::RPC_C_AUTHN_WINNT,
  };
  use crate::windows::pwstr_buf::PwstrBuffer;

  unsafe {
    let mut engine: HANDLE = HANDLE::default();

    let session = FWPM_SESSION0 {
      flags: FWPM_SESSION_FLAG_DYNAMIC,
      ..Default::default()
    };

    let status = FwpmEngineOpen0(
      PWSTR::null(),
      RPC_C_AUTHN_WINNT,
      Some(null()),
      Some(&session),
      &mut engine,
    );
    if status != 0 {
      bail!("FwpmEngineOpen0 failed: {}", status);
    }

    let _engine_guard = scopeguard::guard(engine, |eng| {
      FwpmEngineClose0(eng);
    });

    fwpm_transaction! { engine, {
      let resolver_ip = match resolver.ip() {
          IpAddr::V4(ip) => u32::from_be_bytes(ip.octets()),
          IpAddr::V6(_) => bail!("IPv6 local resolver not supported yet"),
      };

      let permit_cond = [condition_remote_addr_v4(resolver_ip)];
      let mut permit_name = PwstrBuffer::new("Allow local DNS resolver");
      let permit = FilterBuilder::new(
          &mut permit_name,
          FWPM_LAYER_ALE_AUTH_CONNECT_V4,
          FWP_ACTION_PERMIT,
          &permit_cond,
          15,
      ).build();
      add(engine, &permit, "Allow local DNS resolver")?;

      let dns_cond = [condition_remote_port(53)];
      let layers = [
          (FWPM_LAYER_ALE_AUTH_CONNECT_V4, "Block DNS v4"),
          (FWPM_LAYER_ALE_AUTH_CONNECT_V6, "Block DNS v6"),
      ];

      for (layer, label) in layers {
          let mut name = PwstrBuffer::new(label);
          let block = FilterBuilder::new(
              &mut name,
              layer,
              FWP_ACTION_BLOCK,
              &dns_cond,
              14,
          ).build();
          add(engine, &block, label)?;
      }

      Ok(())
  }
    };

    Ok(())
  }
}