use std::ptr::null;

use anyhow::{Result, bail};
use tracing::debug;
use windows::Win32::Foundation::HANDLE;

use crate::{
  Protocol,
  filter::{add, condition_protocol},
  fwpm_transaction,
  wfp_session::WfpSession,
};

pub struct OpenPortsConfig {
  pub dns_port: u16,
  pub doh_port: u16,
}

pub fn open_ports(config: OpenPortsConfig) -> Result<WfpSession> {
  use windows::{
    Win32::{
      NetworkManagement::WindowsFilteringPlatform::{
        FWP_ACTION_PERMIT, FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4,
        FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6, FWPM_SESSION_FLAG_DYNAMIC, FWPM_SESSION0,
        FwpmEngineOpen0, FwpmTransactionAbort0, FwpmTransactionBegin0,
        FwpmTransactionCommit0,
      },
      System::Rpc::RPC_C_AUTHN_WINNT,
    },
    core::PWSTR,
  };

  use crate::{
    filter::{FilterBuilder, condition_local_port},
    pwstr_buf::PwstrBuffer,
  };

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
