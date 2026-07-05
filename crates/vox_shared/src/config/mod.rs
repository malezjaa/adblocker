use crate::config::rewrite::Rewrite;
use crate::config::rules::Rule;
use fs_err::{create_dir_all, read, write};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;
use tracing::{debug, warn};

pub mod rewrite;
pub mod rules;

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
#[serde(deny_unknown_fields)]
pub struct Config {
  pub blocklists: Vec<String>,
  pub rules: Option<Vec<Rule>>,
  #[serde(default)]
  pub doh: DoHConfig,
  #[serde(default)]
  pub dns: DNSConfig,
  #[serde(default = "default_true")]
  pub dashboard: bool,
  pub rewrites: Option<Vec<Rewrite>>,
  #[serde(default)]
  pub resolver: ResolverConfig,
  #[serde(default)]
  pub certs: Certs,
  #[serde(default)]
  pub firewall: FirewallConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DoHConfig {
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default = "default_doh_port")]
  pub port: u16,
}

impl Default for DoHConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      port: default_doh_port(),
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DNSConfig {
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default = "default_dns_port")]
  pub port: u16,
}

impl Default for DNSConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      port: default_dns_port(),
    }
  }
}

fn default_doh_port() -> u16 {
  443
}

fn default_dns_port() -> u16 {
  53
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ResolverConfig {
  #[serde(default)]
  pub dnssec: bool,
  #[serde(default = "default_upstreams")]
  pub upstreams: Vec<UpstreamServer>,
}

impl Default for ResolverConfig {
  fn default() -> Self {
    Self { dnssec: false, upstreams: default_upstreams() }
  }
}

fn default_upstreams() -> Vec<UpstreamServer> {
  vec![
    UpstreamServer::new("cloudflare-dns.com", IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))),
    UpstreamServer::new("cloudflare-dns.com", IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1))),
  ]
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct FirewallConfig {
  #[serde(default = "default_open_ports")]
  /// Automatically opens ports for DoH and DNS
  pub open_ports: bool,
}

fn default_open_ports() -> bool {
  false
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Certs {
  #[serde(default)]
  pub use_local_certificates: bool,
}

fn default_true() -> bool {
  true
}

impl Config {
  pub fn default_values() -> anyhow::Result<Self> {
    Ok(Self {
      blocklists: vec!["oisd-big".into()],
      rules: None,
      doh: Default::default(),
      dns: Default::default(),
      dashboard: true,
      rewrites: None,
      resolver: ResolverConfig::default(),
      certs: Certs::default(),
      firewall: FirewallConfig::default(),
    })
  }

  pub fn compile_regexes(&mut self) -> anyhow::Result<()> {
    if let Some(rewrites) = &mut self.rewrites {
      for rewrite in rewrites {
        rewrite.compile()?;
      }
      debug!("compiled regexes")
    }
    Ok(())
  }

  pub fn from_file<P: AsRef<Path>>(file: P) -> anyhow::Result<Self> {
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
    if config.certs.use_local_certificates {
      warn!("Using locally generated certificates is still experimental!")
    }
    Ok(config)
  }
}