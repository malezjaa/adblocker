use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CertsConfig {
  #[serde(default)]
  pub strategy: CertificateStrategy,

  #[serde(default)]
  pub acme: AcmeConfig,

  #[serde(default)]
  pub manual: ManualCertConfig,
}

impl CertsConfig {
  pub fn self_signed(&self) -> bool {
    matches!(self.strategy, CertificateStrategy::SelfSigned)
  }
}

impl Default for CertsConfig {
  fn default() -> Self {
    Self {
      strategy: CertificateStrategy::SelfSigned,
      acme: AcmeConfig::default(),
      manual: ManualCertConfig::default(),
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum CertificateStrategy {
  #[serde(rename = "acme")]
  #[default]
  Acme,

  #[serde(rename = "self-signed")]
  SelfSigned,

  #[serde(rename = "manual")]
  Manual,

  #[serde(rename = "none")]
  None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AcmeConfig {
  #[serde(default = "default_acme_directory")]
  pub directory_url: String,

  pub email: Option<String>,

  #[serde(default)]
  pub challenge: AcmeChallenge,
  pub domain: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum AcmeChallenge {
  #[serde(rename = "http-01")]
  Http01,

  #[serde(rename = "dns-01")]
  #[default]
  Dns01,

  #[serde(rename = "tls-alpn-01")]
  TlsAlpn01,
}

impl Default for AcmeConfig {
  fn default() -> Self {
    Self {
      directory_url: default_acme_directory(),
      email: None,
      challenge: AcmeChallenge::Dns01,
      domain: None,
    }
  }
}

fn default_acme_directory() -> String {
  "https://acme-v02.api.letsencrypt.org/directory".into()
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ManualCertConfig {
  pub cert_path: Option<String>,
  pub key_path: Option<String>,
}
