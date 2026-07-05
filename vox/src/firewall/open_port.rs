use crate::firewall::Protocol;
use crate::windows::filter::{add, condition_protocol};
use anyhow::{bail, Result};
use std::ptr::null;
use tracing::debug;

use vox_shared::config::Config;
use windows::Win32::Foundation::HANDLE;

#[cfg(windows)]
pub fn open_ports(config: &Config, mut engine: HANDLE) -> Result<()> {
  use crate::fwpm_transaction;
  use crate::windows::filter::{condition_local_port, FilterBuilder};
  use crate::windows::pwstr_buf::PwstrBuffer;

  use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4, FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6,
    FWP_ACTION_PERMIT,
  };
  use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FwpmEngineOpen0, FWPM_SESSION0, FWPM_SESSION_FLAG_DYNAMIC,
  };
  use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FwpmTransactionAbort0, FwpmTransactionBegin0, FwpmTransactionCommit0,
  };
  use windows::Win32::System::Rpc::RPC_C_AUTHN_WINNT;
  use windows::core::PWSTR;

  unsafe {
    let session =
      FWPM_SESSION0 { flags: FWPM_SESSION_FLAG_DYNAMIC, ..Default::default() };

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

    let rules = &[
      (Protocol::UDP, config.dns.port),
      (Protocol::TCP, config.dns.port),
      (Protocol::TCP, config.doh.port),
    ];

    fwpm_transaction! { engine, {
      for (protocol, port) in rules {
        for layer in [FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4, FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6] {
          let label = format!("ADB Local rule: {:?}: {port}", protocol);
          let mut name = PwstrBuffer::new(&label);
          let conditions = &[condition_local_port(*port), condition_protocol(*protocol)];
          let mut filter =
            FilterBuilder::new(&mut name, layer, FWP_ACTION_PERMIT, conditions, 10);
          add(engine, &filter.build(), &label)?;

          debug!("added open port: {label}");
        }
      }

      Ok(())
    }};
  }

  Ok(())
}
