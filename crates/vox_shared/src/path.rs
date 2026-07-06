use std::path::{Path, PathBuf};
use anyhow::Result;

pub fn canonicalize_with_strip<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
  let canonical = fs_err::canonicalize(path)?;
  Ok(strip_windows_long_path_prefix(canonical))
}

fn strip_windows_long_path_prefix(path: PathBuf) -> PathBuf {
  if cfg!(windows) {
    let path_str = path.to_string_lossy();

    if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
      PathBuf::from(stripped)
    } else {
      path
    }
  } else {
    path
  }
}
