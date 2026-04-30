use crate::fwpm_transaction;
use anyhow::{bail, Result};
use std::net::{IpAddr, SocketAddr};
use std::ptr::null;

#[cfg(windows)]
pub fn block_external_dns(resolver: SocketAddr) -> Result<()> {
  use windows::{
    core::PWSTR,
    Win32::Foundation::HANDLE,
    Win32::NetworkManagement::WindowsFilteringPlatform::*,
  };
  use windows::Win32::NetworkManagement::WindowsFilteringPlatform::FWPM_FILTER0;
  use windows::Win32::System::Rpc::RPC_C_AUTHN_WINNT;
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
      let resolver_ip_bytes = match resolver.ip() {
        IpAddr::V4(ip) => ip.octets(),
        IpAddr::V6(_) => bail!("IPv6 local resolver not supported yet"),
      };

      let permit_condition = FWPM_FILTER_CONDITION0 {
        fieldKey: FWPM_CONDITION_IP_REMOTE_ADDRESS,
        matchType: FWP_MATCH_EQUAL,
        conditionValue: FWP_CONDITION_VALUE0 {
          r#type: FWP_UINT32,
          Anonymous: FWP_CONDITION_VALUE0_0 {
            uint32: u32::from_be_bytes(resolver_ip_bytes),
          },
        },
      };

      let permit_conditions = [permit_condition];
      let permit_name = PwstrBuffer::new("Allow local DNS resolver");

      let mut permit_filter = FWPM_FILTER0::default();
      permit_filter.displayData.name = permit_name.as_pwstr();
      permit_filter.layerKey = FWPM_LAYER_ALE_AUTH_CONNECT_V4;
      permit_filter.action.r#type = FWP_ACTION_PERMIT;
      permit_filter.numFilterConditions = permit_conditions.len() as u32;
      permit_filter.filterCondition = permit_conditions.as_ptr() as *mut _;
      permit_filter.weight.r#type = FWP_UINT8;
      permit_filter.weight.Anonymous = FWP_VALUE0_0 { uint8: 15 };

      let status = FwpmFilterAdd0(engine, &permit_filter, None, None);
      if status != 0 {
        bail!("Failed to add local resolver permit filter: {:#010x}", status);
      }

      let layers = [
        (FWPM_LAYER_ALE_AUTH_CONNECT_V4, "Block DNS v4"),
        (FWPM_LAYER_ALE_AUTH_CONNECT_V6, "Block DNS v6"),
      ];

      for (layer, label) in layers {
        let condition = FWPM_FILTER_CONDITION0 {
          fieldKey: FWPM_CONDITION_IP_REMOTE_PORT,
          matchType: FWP_MATCH_EQUAL,
          conditionValue: FWP_CONDITION_VALUE0 {
            r#type: FWP_UINT16,
            Anonymous: FWP_CONDITION_VALUE0_0 { uint16: 53 },
          },
        };

        let conditions = [condition];
        let name = PwstrBuffer::new(label);

        let mut filter = FWPM_FILTER0::default();
        filter.displayData.name = name.as_pwstr();
        filter.layerKey = layer;
        filter.action.r#type = FWP_ACTION_BLOCK;
        filter.numFilterConditions = conditions.len() as u32;
        filter.filterCondition = conditions.as_ptr() as *mut _;

        filter.weight.r#type = FWP_UINT8;
        filter.weight.Anonymous = FWP_VALUE0_0 { uint8: 14 };

        let status = FwpmFilterAdd0(engine, &filter, None, None);
        if status != 0 {
          bail!("Failed to add DNS block filter ({}): {:#010x}", label, status);
        }
      }

      Ok(())
    }};

    Ok(())
  }
}