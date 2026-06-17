use bitflags::bitflags;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Compatibility {
  Safe,
  Balanced,
  Aggressive,
}

bitflags! {
  #[derive(Debug, Clone, Copy, Serialize)]
  pub struct Categories: u16 {
    const ADS       = 1 << 0;
    const PRIVACY   = 1 << 1;
    const SECURITY  = 1 << 2;
    const NSFW      = 1 << 3;
    const GAMBLING  = 1 << 4;
    const FAKE_NEWS = 1 << 5;
  }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct List {
  pub id: &'static str,
  pub name: &'static str,
  pub description: &'static str,
  pub homepage: &'static str,
  pub url: &'static str,

  pub categories: Categories,
  pub compatibility: Compatibility,

  pub recommended: bool,
  pub default_enabled: bool,
  pub priority: u16,

  pub domains: Option<usize>,
  pub enabled: Option<bool>,
}

macro_rules! define_lists {
  ($(
    $name:literal: $id:literal => $desc:literal {
      home => $home:literal,
      url => $url:literal,

      categories => $categories:expr,
      compatibility => $compat:ident,

      recommended => $recommended:literal,
      default_enabled => $default_enabled:literal,
      priority => $priority:literal
    }
  )*) => {
    pub const LISTS: &[List] = &[
      $(
        List {
          id: $id,
          name: $name,
          description: $desc,
          homepage: $home,
          url: $url,

          categories: $categories,
          compatibility: Compatibility::$compat,

          recommended: $recommended,
          default_enabled: $default_enabled,
          priority: $priority,

          domains: None,
          enabled: None
        },
      )*
    ];

    pub const LIST_IDS: &[&str] = &[
      $($id,)*
    ];
  };
}

/// We use this in our macro because BitOr can't be made const.
macro_rules! cats {
    ($first:ident $(| $rest:ident)*) => {
        Categories::$first $(.union(Categories::$rest))*
    };
}

define_lists! {
  "OISD Big": "oisd-big" => "Blocks ads, trackers, malware, phishing, telemetry, mobile app ads and many nuisance domains." {
    home => "https://oisd.nl",
    url => "https://big.oisd.nl",

    categories => cats!(ADS | PRIVACY | SECURITY),
    compatibility => Balanced,

    recommended => true,
    default_enabled => true,
    priority => 100
  }

  "OISD Small": "oisd-small" => "Smaller OISD variant focused on ads and trackers with fewer false positives." {
    home => "https://oisd.nl",
    url => "https://small.oisd.nl",

    categories => cats!(ADS | PRIVACY),
    compatibility => Safe,

    recommended => true,
    default_enabled => false,
    priority => 90
  }

  "StevenBlack Hosts": "stevenblack" => "Popular consolidated hosts file combining multiple reputable ad and tracking blocklists." {
    home => "https://github.com/StevenBlack/hosts",
    url => "https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts",

    categories => cats!(ADS | PRIVACY),
    compatibility => Balanced,

    recommended => false,
    default_enabled => false,
    priority => 80
  }

  "HaGeZi Multi PRO": "hagezi-pro" => "Comprehensive protection against ads, tracking, telemetry, phishing and malware while maintaining good compatibility." {
    home => "https://github.com/hagezi/dns-blocklists",
    url => "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/pro.txt",

    categories => cats!(ADS | PRIVACY | SECURITY),
    compatibility => Balanced,

    recommended => true,
    default_enabled => false,
    priority => 95
  }

  "HaGeZi Multi ULTIMATE": "hagezi-ultimate" => "Aggressive blocklist with maximum coverage of ads, tracking, telemetry and unwanted domains." {
    home => "https://github.com/hagezi/dns-blocklists",
    url => "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/ultimate.txt",

    categories => cats!(ADS | PRIVACY | SECURITY),
    compatibility => Aggressive,

    recommended => false,
    default_enabled => false,
    priority => 70
  }

  "HaGeZi Threat Intelligence": "hagezi-tif" => "Blocks known malware, botnet, phishing and command-and-control domains." {
    home => "https://github.com/hagezi/dns-blocklists",
    url => "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/tif.txt",

    categories => Categories::SECURITY,
    compatibility => Safe,

    recommended => true,
    default_enabled => true,
    priority => 110
  }

  "AdGuard DNS Filter": "adguard-dns" => "Official AdGuard DNS blocklist targeting ads, trackers and malicious domains." {
    home => "https://github.com/AdguardTeam/AdguardSDNSFilter",
    url => "https://adguardteam.github.io/HostlistsRegistry/assets/filter_1.txt",

    categories => cats!(ADS | PRIVACY),
    compatibility => Balanced,

    recommended => false,
    default_enabled => false,
    priority => 75
  }

  "AdGuard Tracking Protection": "adguard-tracking" => "Focused on analytics, telemetry and user tracking domains." {
    home => "https://github.com/AdguardTeam/AdguardSDNSFilter",
    url => "https://adguardteam.github.io/HostlistsRegistry/assets/filter_3.txt",

    categories => Categories::PRIVACY,
    compatibility => Safe,

    recommended => false,
    default_enabled => false,
    priority => 65
  }

  "EasyList": "easylist" => "The most widely used community-maintained advertising filter list." {
    home => "https://easylist.to",
    url => "https://easylist.to/easylist/easylist.txt",

    categories => Categories::ADS,
    compatibility => Balanced,

    recommended => false,
    default_enabled => false,
    priority => 60
  }

  "EasyPrivacy": "easyprivacy" => "Companion list to EasyList focused on tracking and analytics domains." {
    home => "https://easylist.to",
    url => "https://easylist.to/easylist/easyprivacy.txt",

    categories => Categories::PRIVACY,
    compatibility => Safe,

    recommended => false,
    default_enabled => false,
    priority => 55
  }

  "URLHaus Malware": "urlhaus" => "Threat intelligence feed containing domains associated with malware distribution." {
    home => "https://urlhaus.abuse.ch",
    url => "https://urlhaus.abuse.ch/downloads/hostfile/",

    categories => Categories::SECURITY,
    compatibility => Safe,

    recommended => true,
    default_enabled => false,
    priority => 105
  }

  "Phishing Army": "phishing-army" => "Community-maintained phishing domain blocklist." {
    home => "https://phishing.army",
    url => "https://phishing.army/download/phishing_army_blocklist_extended.txt",

    categories => Categories::SECURITY,
    compatibility => Safe,

    recommended => true,
    default_enabled => false,
    priority => 100
  }

  "Dandelion Sprout Anti-Malware": "dandelion-malware" => "Additional malware, scam and malicious domain protection." {
    home => "https://github.com/DandelionSprout/adfilt",
    url => "https://raw.githubusercontent.com/DandelionSprout/adfilt/master/Alternate%20versions%20Anti-Malware%20List/AntiMalwareDomains.txt",

    categories => Categories::SECURITY,
    compatibility => Safe,

    recommended => false,
    default_enabled => false,
    priority => 50
  }

  "OISD NSFW": "oisd-nsfw" => "Blocks adult and explicit content domains." {
    home => "https://oisd.nl",
    url => "https://nsfw.oisd.nl",

    categories => Categories::NSFW,
    compatibility => Safe,

    recommended => false,
    default_enabled => false,
    priority => 50
  }

  "HaGeZi NSFW": "hagezi-nsfw" => "Blocks adult and explicit content domains." {
    home => "https://github.com/hagezi/dns-blocklists",
    url => "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/nsfw.txt",

    categories => Categories::NSFW,
    compatibility => Safe,

    recommended => false,
    default_enabled => false,
    priority => 45
  }

  "HaGeZi Gambling": "hagezi-gambling" => "Blocks gambling and betting websites." {
    home => "https://github.com/hagezi/dns-blocklists",
    url => "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/gambling.txt",

    categories => Categories::GAMBLING,
    compatibility => Safe,

    recommended => false,
    default_enabled => false,
    priority => 40
  }

  "HaGeZi Fake": "hagezi-fake" => "Blocks fake shops, scams and misleading websites." {
    home => "https://github.com/hagezi/dns-blocklists",
    url => "https://raw.githubusercontent.com/hagezi/dns-blocklists/main/adblock/fake.txt",

    categories => cats!(FAKE_NEWS | SECURITY),
    compatibility => Safe,

    recommended => false,
    default_enabled => false,
    priority => 45
  }
}

pub fn default_enabled_lists() -> Vec<&'static List> {
  LISTS.iter().filter(|l| l.default_enabled).collect()
}

pub fn recommended_lists() -> Vec<&'static List> {
  LISTS.iter().filter(|l| l.recommended).collect()
}

pub fn privacy_lists() -> Vec<&'static List> {
  by_category(Categories::PRIVACY)
}

pub fn security_lists() -> Vec<&'static List> {
  by_category(Categories::SECURITY)
}

pub fn nsfw_lists() -> Vec<&'static List> {
  by_category(Categories::NSFW)
}

pub fn gambling_lists() -> Vec<&'static List> {
  by_category(Categories::GAMBLING)
}

pub fn by_category(category: Categories) -> Vec<&'static List> {
  LISTS.iter().filter(|l| l.categories.contains(category)).collect()
}

pub fn get_list(id: &str) -> Option<&'static List> {
  LISTS.iter().find(|l| l.id == id)
}
