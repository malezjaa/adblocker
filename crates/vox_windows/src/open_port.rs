use crate::filter::{add, condition_protocol};
use crate::wfp_session::WfpSession;
use crate::{Protocol, fwpm_transaction};
use anyhow::{Result, bail};
use std::ptr::null;
use tracing::debug;
use windows::Win32::Foundation::HANDLE;

pub struct OpenPortsConfig {
  pub dns_port: u16,
  pub doh_port: u16,
}

pub fn open_ports(config: OpenPortsConfig) -> Result<WfpSession> {
  use crate::filter::{FilterBuilder, condition_local_port};
  use crate::pwstr_buf::PwstrBuffer;

  use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FWP_ACTION_PERMIT, FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4,
    FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6,
  };
  use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FWPM_SESSION_FLAG_DYNAMIC, FWPM_SESSION0, FwpmEngineOpen0,
  };
  use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FwpmTransactionAbort0, FwpmTransactionBegin0, FwpmTransactionCommit0,
  };
  use windows::Win32::System::Rpc::RPC_C_AUTHN_WINNT;
  use windows::core::PWSTR;

  let mut engine = HANDLE::default();

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
      (Protocol::UDP, config.dns_port),
      (Protocol::TCP, config.dns_port),
      (Protocol::TCP, config.doh_port),
    ];

    fwpm_transaction! { engine, {
      for (protocol, port) in rules {
        for layer in [FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4, FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6] {
          let label = format!("Vox Local rule: {:?}: {port}", protocol);
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

  Ok(WfpSession { engine })
}
