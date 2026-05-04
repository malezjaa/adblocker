use anyhow::Result;
use fs_err::{create_dir, read, write};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;
use tracing::info;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
  pub blocklists: Vec<String>,
  pub socket: SocketAddr,
}

impl Config {
  pub fn default_values() -> Result<Self> {
    Ok(Self {
      blocklists: vec!["https://big.oisd.nl".into()],
      socket: "127.0.0.2:53".parse()?,
    })
  }

  pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Self> {
    let path = file.as_ref();
    info!(path = path.display().to_string(), "loading config");

    if !path.exists() {
      let config = Self::default_values()?;
      if let Some(parent) = path.parent() {
        create_dir(parent)?;
      }

      write(path, toml::to_string(&config)?)?;
      return Ok(config);
    }

    let content = read(path)?;
    Ok(toml::from_slice(&content)?)
  }
}
