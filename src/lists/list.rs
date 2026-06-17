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
  "OISD Big": "oisd-big" => "Blocks ads, trackers, malware, phishing, telemetry, mobile app ads and many nuisance domains." {
    home => "https://oisd.nl",
    url => "https://big.oisd.nl"
  }

  "OISD Small": "oisd-small" => "Smaller OISD variant focused on ads and trackers with fewer false positives." {
    home => "https://oisd.nl",
    url => "https://small.oisd.nl"
  }

  "StevenBlack Hosts": "stevenblack" => "Popular consolidated hosts file combining multiple reputable ad and tracking blocklists." {
    home => "https://github.com/StevenBlack/hosts",
    url => "https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts"
  }

  "Hagezi Multi PRO": "hagezi-pro" => "Comprehensive protection against ads, tracking, telemetry, phishing and malware while maintaining good compatibility." {
    home => "https://github.com/hagezi/dns-blocklists",
    url => "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/pro.txt"
  }

  "Hagezi Multi ULTIMATE": "hagezi-ultimate" => "Aggressive blocklist with maximum coverage of ads, tracking, telemetry and unwanted domains." {
    home => "https://github.com/hagezi/dns-blocklists",
    url => "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/ultimate.txt"
  }

  "Hagezi Threat Intelligence": "hagezi-tif" => "Blocks known malware, botnet, phishing and command-and-control domains." {
    home => "https://github.com/hagezi/dns-blocklists",
    url => "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/tif.txt"
  }

  "AdGuard DNS Filter": "adguard-dns" => "Official AdGuard DNS blocklist targeting ads, trackers and malicious domains." {
    home => "https://github.com/AdguardTeam/AdguardSDNSFilter",
    url => "https://adguardteam.github.io/HostlistsRegistry/assets/filter_1.txt"
  }

  "AdGuard Tracking Protection": "adguard-tracking" => "Focused on analytics, telemetry and user tracking domains." {
    home => "https://github.com/AdguardTeam/AdguardSDNSFilter",
    url => "https://adguardteam.github.io/HostlistsRegistry/assets/filter_3.txt"
  }

  "EasyList": "easylist" => "The most widely used community-maintained advertising filter list." {
    home => "https://easylist.to",
    url => "https://easylist.to/easylist/easylist.txt"
  }

  "EasyPrivacy": "easyprivacy" => "Companion list to EasyList focused on tracking and analytics domains." {
    home => "https://easylist.to",
    url => "https://easylist.to/easylist/easyprivacy.txt"
  }

  "URLHaus Malware": "urlhaus" => "Threat intelligence feed containing domains associated with malware distribution." {
    home => "https://urlhaus.abuse.ch",
    url => "https://urlhaus.abuse.ch/downloads/hostfile/"
  }

  "Phishing Army": "phishing-army" => "Community-maintained phishing domain blocklist." {
    home => "https://phishing.army",
    url => "https://raw.githubusercontent.com/DandelionSprout/adfilt/master/Alternate%20versions%20Anti-Malware%20List/AntiMalwareHosts.txt"
  }

  "Dandelion Sprout Anti-Malware": "dandelion-malware" => "Additional malware, scam and malicious domain protection." {
    home => "https://github.com/DandelionSprout/adfilt",
    url => "https://raw.githubusercontent.com/DandelionSprout/adfilt/master/Alternate%20versions%20Anti-Malware%20List/AntiMalwareDomains.txt"
  }
}