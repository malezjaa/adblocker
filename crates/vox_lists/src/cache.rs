use chrono::{DateTime, Duration, Utc};
use fs_err::{create_dir_all, read, write};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
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
  debug!(path = path.display().to_string(), "loading config");

  if !path.exists() {
    let cache = CacheFile::default();
    if let Some(parent) = path.parent() {
      create_dir_all(parent)?;
    }

    write(path, toml::to_string(&cache)?)?;
    return Ok(cache);
  }

  let content = read(path)?;
  Ok(toml::from_slice(&content)?)
}
