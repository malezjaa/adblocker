use std::{collections::HashMap, path::Path};

use chrono::{DateTime, Duration, Utc};
use fs_err::{create_dir_all, read, write};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::debug;

#[derive(Serialize, Deserialize, Default)]
pub struct CacheFile {
  pub lists: HashMap<String, ListEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct ListEntry {
  pub id: String,
  pub last_fetched: DateTime<Utc>,
  pub etag: Option<String>,
  pub domains: usize,
}

impl CacheFile {
  pub fn id_hash(id: &str) -> String {
    let hash = Sha256::digest(id.as_bytes());

    hash.iter().take(8).map(|b| format!("{:02x}", b)).collect()
  }

  pub fn get_by_id(&self, id: &str) -> Option<&ListEntry> {
    self.lists.get(&Self::id_hash(id))
  }

  pub fn is_fresh(&self, id: &str, max_age: Duration) -> bool {
    self.get_by_id(id).map(|e| Utc::now() - e.last_fetched < max_age).unwrap_or(false)
  }

  pub fn insert(&mut self, id: &str, etag: Option<String>, domains: usize) {
    self.lists.insert(
      Self::id_hash(id),
      ListEntry { id: id.to_string(), last_fetched: Utc::now(), etag, domains },
    );
  }
}

pub fn load_cache_file(cache_dir: &Path) -> anyhow::Result<CacheFile> {
  let path = cache_dir.join("cache.toml");
  debug!(path = path.display().to_string(), "loading cache file");

  if !path.exists() {
    let cache = CacheFile::default();
    if let Some(parent) = path.parent() {
      create_dir_all(parent)?;
    }

    write(path, toml::to_string_pretty(&cache)?)?;
    return Ok(cache);
  }

  let content = read(path)?;
  Ok(toml::from_slice(&content)?)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn insert_stores_entries_by_stable_id_hash() {
    let mut cache = CacheFile::default();
    let id = "https://example.test/blocklist.txt";

    cache.insert(id, Some("\"etag-1\"".into()), 42);

    assert!(!cache.lists.contains_key(id));

    let entry = cache.get_by_id(id).unwrap();
    assert_eq!(entry.id, id);
    assert_eq!(entry.etag.as_deref(), Some("\"etag-1\""));
    assert_eq!(entry.domains, 42);
  }

  #[test]
  fn freshness_depends_on_entry_age_and_requested_max_age() {
    let mut cache = CacheFile::default();
    let id = "https://example.test/blocklist.txt";

    cache.lists.insert(
      CacheFile::id_hash(id),
      ListEntry {
        id: id.into(),
        last_fetched: Utc::now() - Duration::hours(2),
        etag: None,
        domains: 10,
      },
    );

    assert!(cache.is_fresh(id, Duration::hours(3)));
    assert!(!cache.is_fresh(id, Duration::hours(1)));
    assert!(!cache.is_fresh("https://example.test/missing.txt", Duration::hours(3)));
  }
}
