use std::os::windows::raw::HANDLE;

pub mod adapters;
pub mod filter;
pub mod primary_adapter;
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

pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
