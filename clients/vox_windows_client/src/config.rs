use std::{net::SocketAddr, path::Path};

use anyhow::bail;
use fs_err::{create_dir_all, read, write};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WinClientConfig {
  pub dns_server: SocketAddr,
  pub doh: Option<SocketAddr>,
}

impl WinClientConfig {
  pub fn from_file<P: AsRef<Path>>(file: P) -> anyhow::Result<Self> {
    let path = file.as_ref();
    debug!(path = path.display().to_string(), "loading windows client config");

    if !path.exists() {
      if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
      }

      write(path, String::new())?;
      bail!("created config. please fill in required fields.");
    }

    Ok(toml::from_slice(&read(path)?)?)
  }

  pub fn using_doh(&self) -> bool {
    self.doh.is_some()
  }
}
