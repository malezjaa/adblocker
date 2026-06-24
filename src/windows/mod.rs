pub mod adapters;
pub mod cert_store;
pub mod filter;
pub mod pwstr_buf;
pub mod transaction;

#[cfg(windows)]
pub mod wfp_session {
  use windows::Win32::Foundation::HANDLE;

  #[derive(Clone)]
  pub struct WfpSession {
    pub engine: HANDLE,
  }
}
