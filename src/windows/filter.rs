use crate::windows::pwstr_buf::PwstrBuffer;
use anyhow::{Result, bail};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::NetworkManagement::WindowsFilteringPlatform::*;
use windows::core::GUID;

pub fn condition_remote_addr_v4(ip: u32) -> FWPM_FILTER_CONDITION0 {
  FWPM_FILTER_CONDITION0 {
    fieldKey: FWPM_CONDITION_IP_REMOTE_ADDRESS,
    matchType: FWP_MATCH_EQUAL,
    conditionValue: FWP_CONDITION_VALUE0 {
      r#type: FWP_UINT32,
      Anonymous: FWP_CONDITION_VALUE0_0 { uint32: ip },
    },
  }
}

pub fn condition_remote_port(port: u16) -> FWPM_FILTER_CONDITION0 {
  FWPM_FILTER_CONDITION0 {
    fieldKey: FWPM_CONDITION_IP_REMOTE_PORT,
    matchType: FWP_MATCH_EQUAL,
    conditionValue: FWP_CONDITION_VALUE0 {
      r#type: FWP_UINT16,
      Anonymous: FWP_CONDITION_VALUE0_0 { uint16: port },
    },
  }
}

pub struct FilterBuilder<'a> {
  name: &'a mut PwstrBuffer,
  layer: GUID,
  action: FWP_ACTION_TYPE,
  conditions: &'a [FWPM_FILTER_CONDITION0],
  weight: u8,
}

impl<'a> FilterBuilder<'a> {
  pub fn new(
    name: &'a mut PwstrBuffer,
    layer: GUID,
    action: FWP_ACTION_TYPE,
    conditions: &'a [FWPM_FILTER_CONDITION0],
    weight: u8,
  ) -> Self {
    Self { name, layer, action, conditions, weight }
  }

  pub fn build(&mut self) -> FWPM_FILTER0 {
    let mut f = FWPM_FILTER0::default();
    f.displayData.name = self.name.as_pwstr();
    f.layerKey = self.layer;
    f.action.r#type = self.action;
    f.numFilterConditions = self.conditions.len() as u32;
    f.filterCondition = self.conditions.as_ptr() as *mut _;
    f.weight.r#type = FWP_UINT8;
    f.weight.Anonymous = FWP_VALUE0_0 { uint8: self.weight };
    f
  }
}

pub unsafe fn add(engine: HANDLE, filter: &FWPM_FILTER0, label: &str) -> Result<()> {
  let status = unsafe { FwpmFilterAdd0(engine, filter, None, None) };
  if status != 0 {
    bail!("Failed to add filter '{}': {:#010x}", label, status);
  }
  Ok(())
}
