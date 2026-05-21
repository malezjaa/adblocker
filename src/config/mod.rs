pub mod watcher;

use crate::rewrite::Rewrite;
use anyhow::Result;
use fs_err::{create_dir, read, write};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;
use tracing::debug;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
  pub blocklists: Vec<String>,
  pub secondary_name_server: Option<SocketAddr>,
  pub block_rules: Option<Vec<String>>,
  pub doh: Option<bool>,
  pub dot: Option<bool>,
  pub dashboard: Option<bool>,
  pub rewrites: Option<Vec<Rewrite>>,
}

impl Config {
  pub fn default_values() -> Result<Self> {
    Ok(Self {
      blocklists: vec!["https://big.oisd.nl".into()],
      secondary_name_server: None,
      block_rules: None,
      doh: Some(true),
      dot: Some(true),
      dashboard: Some(true),
      rewrites: None,
    })
  }

  pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Self> {
    let path = file.as_ref();
    debug!(path = path.display().to_string(), "loading config");

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

  pub fn doh_enabled(&self) -> bool {
    self.doh.unwrap_or(true)
  }

  pub fn dot_enabled(&self) -> bool {
    self.dot.unwrap_or(true)
  }

  pub fn dashboard_enabled(&self) -> bool {
    self.dashboard.unwrap_or(true)
  }
}
