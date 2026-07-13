use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rewrite {
  pub name: Option<String>,
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default = "default_priority")]
  pub priority: i32,
  pub when: RewriteMatchWhen,
  #[serde(default)]
  pub conditions: RewriteConditions,
  pub behavior: RewriteBehavior,
  #[serde(default)]
  pub ttl: Option<u32>,
  #[serde(default)]
  pub continue_processing: bool,
  #[serde(skip)]
  pub regex: Option<Regex>,
}

impl Rewrite {
  pub fn compile(&mut self) -> anyhow::Result<()> {
    if self.when.ty == RewriteMatchWhenType::Regex {
      self.regex =
        Some(RegexBuilder::new(&self.when.value).case_insensitive(true).build()?);
    }
    Ok(())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteMatchWhen {
  #[serde(rename = "type")]
  pub ty: RewriteMatchWhenType,
  pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RewriteMatchWhenType {
  Exact,
  Suffix,
  Wildcard,
  Regex,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RewriteConditions {
  /// Limit this rule to specific query types. Empty means every query type.
  #[serde(default)]
  pub query_types: Vec<RewriteRecordType>,
  /// Limit this rule to specific dashboard/client device names or identifiers.
  #[serde(default)]
  pub devices: Vec<String>,
  /// Limit this rule to specific DNS transports. Empty means every transport.
  #[serde(default)]
  pub transports: Vec<RewriteTransportCondition>,
  /// Limit this rule to specific client operating systems. Empty means every
  /// client OS.
  #[serde(default)]
  pub client_origins: Vec<RewriteClientCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RewriteTransportCondition {
  Plain,
  #[serde(rename = "doh")]
  DoH,
  #[serde(rename = "dot")]
  DoT,
  #[serde(rename = "doq")]
  DoQ,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RewriteClientCondition {
  Windows,
  Linux,
  #[serde(alias = "macos")]
  Mac,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RewriteBehavior {
  /// Return synthetic DNS records without resolving upstream.
  Respond { records: Vec<RewriteRecord>, ttl: Option<u32> },
  /// Return an alias record for the queried name.
  Alias { target: String, ttl: Option<u32> },
  /// Resolve a different DNS name while preserving the original query name in
  /// the response.
  Forward { target: String },
  /// Respond with NXDOMAIN.
  NxDomain,
  /// Respond with empty NOERROR.
  NoData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteRecord {
  #[serde(rename = "type")]
  pub ty: RewriteRecordType,
  pub value: RewriteRecordValue,
  #[serde(default)]
  pub ttl: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RewriteRecordType {
  A,
  AAAA,
  CNAME,
  MX,
  TXT,
  PTR,
  SRV,
  HTTPS,
  SVCB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum RewriteRecordValue {
  A { value: String },
  AAAA { value: String },
  CNAME { value: String },
  MX { exchange: String, preference: u16 },
  TXT { value: Vec<String> },
  PTR { value: String },
  SRV { priority: u16, weight: u16, port: u16, target: String },
  HTTPS { priority: u16, target: String, params: Vec<String> },
  SVCB { priority: u16, target: String, params: Vec<String> },
}

fn default_true() -> bool {
  true
}

fn default_priority() -> i32 {
  100
}
