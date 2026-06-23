pub mod rules;
pub mod settings;
pub mod watcher;

use crate::config::rules::Rule;
use crate::rewrite::Rewrite;
use anyhow::Result;
use fs_err::{create_dir, create_dir_all, read, write};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::path::Path;
use tracing::debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpstreamServer {
  pub name: String,
  pub addr: IpAddr,
}

impl UpstreamServer {
  pub fn new(name: impl Into<String>, addr: IpAddr) -> Self {
    Self { name: name.into(), addr }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
  pub blocklists: Vec<String>,
  pub upstreams: Option<Vec<UpstreamServer>>,
  pub rules: Option<Vec<Rule>>,
  pub doh: Option<bool>,
  pub dashboard: Option<bool>,
  pub rewrites: Option<Vec<Rewrite>>,
  pub dnssec: Option<bool>,
}

impl Config {
  pub fn default_values() -> Result<Self> {
    Ok(Self {
      blocklists: vec!["oisd-big".into()],
      upstreams: Some(vec![
        UpstreamServer::new("cloudflare-dns.com", IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))),
        UpstreamServer::new("cloudflare-dns.com", IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1))),
      ]),
      rules: None,
      doh: Some(true),
      dashboard: Some(true),
      rewrites: None,
      dnssec: Some(false),
    })
  }

  pub fn compile_regexes(&mut self) -> Result<()> {
    if let Some(rewrites) = &mut self.rewrites {
      for rewrite in rewrites {
        rewrite.compile()?;
      }
      debug!("compiled regexes")
    }

    Ok(())
  }

  pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Self> {
    let path = file.as_ref();
    debug!(path = path.display().to_string(), "loading config");

    if !path.exists() {
      let config = Self::default_values()?;
      if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
      }

      write(path, toml::to_string(&config)?)?;
      return Ok(config);
    }

    let mut config: Config = toml::from_slice(&read(path)?)?;
    config.compile_regexes()?;
    config.validate_rules();

    Ok(config)
  }

  pub fn doh_enabled(&self) -> bool {
    self.doh.unwrap_or(true)
  }

  pub fn dashboard_enabled(&self) -> bool {
    self.dashboard.unwrap_or(true)
  }

  pub fn dnssec_enabled(&self) -> bool {
    self.dnssec.unwrap_or(false)
  }
}
