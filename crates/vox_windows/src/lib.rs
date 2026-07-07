pub mod adapters;
pub mod filter;
pub mod open_port;
pub mod override_dns;
pub mod primary_adapter;
pub mod pwstr_buf;
pub mod transaction;

#[derive(Clone, Copy, Debug)]
pub enum Protocol {
  TCP,
  UDP,
}

#[cfg(windows)]
pub mod wfp_session {
  use windows::Win32::Foundation::HANDLE;
  use windows::Win32::NetworkManagement::WindowsFilteringPlatform::FwpmEngineClose0;

  pub struct WfpSession {
    pub(crate) engine: HANDLE,
  }

  impl Drop for WfpSession {
    fn drop(&mut self) {
      unsafe {
        FwpmEngineClose0(self.engine);
      }
    }
  }
}

#[cfg(windows)]
pub use wfp_session::WfpSession;
