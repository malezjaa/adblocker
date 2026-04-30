use windows::core::PWSTR;

pub struct PwstrBuffer {
  buf: Vec<u16>,
  ptr: PWSTR,
}

impl PwstrBuffer {
  pub fn new(s: &str) -> Self {
    let mut buf: Vec<u16> = s.encode_utf16().chain(Some(0)).collect();
    let ptr = PWSTR(buf.as_mut_ptr());

    Self { buf, ptr }
  }

  pub fn as_pwstr(&self) -> PWSTR {
    self.ptr
  }
}