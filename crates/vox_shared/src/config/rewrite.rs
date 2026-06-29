use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rewrite {
  pub name: Option<String>,
  pub when: RewriteMatchWhen,
  pub actions: Vec<RewriteAction>,
  #[serde(skip)]
  pub regex: Option<Regex>,
}

impl Rewrite {
  pub fn compile(&mut self) -> anyhow::Result<()> {
    if self.when.ty == RewriteMatchWhenType::Regex {
      self.regex = Some(Regex::new(&self.when.value)?);
    }
    Ok(())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteMatchWhen {
  #[serde(rename = "type")]
  pub ty: RewriteMatchWhenType,
  /// The pattern to match against - a literal domain or a regex string.
  pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RewriteMatchWhenType {
  Exact,
  Regex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum RewriteAction {
  /// Respond with a static IPv4 address.
  A { value: String },
  /// Respond with a static IPv6 address.
  AAAA { value: String },
  /// Alias this name to another; the resolver follows the chain.
  CNAME { value: String },
  /// Mail exchanger record. Lower `preference` = higher priority.
  MX { exchange: String, preference: u16 },
  /// One or more text strings (SPF, DKIM, verification tokens, …).
  TXT { value: Vec<String> },
  /// Reverse-DNS pointer, typically in `.arpa` zones.
  PTR { value: String },
  /// Service-locator record used by SIP, XMPP, and similar protocols.
  SRV { priority: u16, weight: u16, port: u16, target: String },
  /// Transparently rewrite the queried name before resolving.
  #[serde(rename = "rewrite")]
  Rewrite { value: String },
  /// Respond with NXDOMAIN - the name does not exist.
  NXDOMAIN,
  /// Respond with an empty NOERROR - name exists but has no data for this type.
  NOERROR,
}
