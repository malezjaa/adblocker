use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default)]
pub struct CacheFile {
  pub lists: HashMap<String, ListEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct ListEntry {
  pub url: String,
  pub last_fetched: DateTime<Utc>,
  pub etag: Option<String>,
}

impl CacheFile {
  pub fn url_hash(url: &str) -> String {
    let hash = Sha256::digest(url.as_bytes());

    hash.iter().take(8).map(|b| format!("{:02x}", b)).collect()
  }

  pub fn get_by_url(&self, url: &str) -> Option<&ListEntry> {
    self.lists.get(&Self::url_hash(url))
  }

  pub fn is_fresh(&self, url: &str, max_age: Duration) -> bool {
    self.get_by_url(url).map(|e| Utc::now() - e.last_fetched < max_age).unwrap_or(false)
  }

  pub fn insert(&mut self, url: &str, etag: Option<String>) {
    self.lists.insert(
      Self::url_hash(url),
      ListEntry { url: url.to_string(), last_fetched: Utc::now(), etag },
    );
  }
}
