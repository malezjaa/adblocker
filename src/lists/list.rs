use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct List {
  pub id: String,
  pub name: String,
  pub description: String,
  pub homepage: String,
  pub url: String,
  pub domains: Option<usize>,
  pub enabled: Option<bool>,
}

#[macro_export]
macro_rules! define_lists {
  ($($name:literal: $id:literal => $desc:literal {
    home => $home:literal,
    url => $url:literal
  })*) => {
    pub fn default_lists() -> Vec<List> {
      vec![$(
        List {
          id: $id.to_string(),
          name: $name.to_string(),
          description: $desc.to_string(),
          homepage: $home.to_string(),
          url: $url.to_string(),
          domains: None,
          enabled: None
        },
      )*]
    }

    pub const LISTS_IDS: &'static [&'static str] = &[$($id,)*];
  };
}

define_lists! {
  "OISD Big": "oisd-big" => "Blocks Ads, (Mobile) App Ads, Phishing, Malvertising, Malware, Spyware, Ransomware, CryptoJacking, Telemetry/Analytics/Tracking" {
    home => "https://oisd.nl",
    url => "https://big.oisd.nl"
  }
  "OISD Small": "oisd-small" => "Mainly focuses on blocking ads." {
    home => "https://oisd.nl",
    url => "https://small.oisd.nl"
  }
}
