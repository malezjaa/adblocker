use hickory_proto::rr::RecordType;
use moka::sync::Cache;
use std::time::{Duration, Instant};

pub type CacheKey = (String, RecordType);

#[derive(Clone)]
struct CachedResponse {
  expires_at: Instant,
  bytes: Vec<u8>,
}

pub struct ResponseCache {
  cache: Cache<CacheKey, CachedResponse>,
}

impl ResponseCache {
  pub fn new(max_entries: u64) -> Self {
    Self { cache: Cache::builder().max_capacity(max_entries).build() }
  }

  pub fn get_with_id(&self, key: &CacheKey, id: u16) -> Option<Vec<u8>> {
    let now = Instant::now();
    let entry = self.cache.get(key)?;

    if entry.expires_at <= now {
      self.cache.invalidate(key);
      return None;
    }

    let mut bytes = entry.bytes;
    if bytes.len() >= 2 {
      bytes[0] = (id >> 8) as u8;
      bytes[1] = (id & 0xff) as u8;
    }

    Some(bytes)
  }

  pub fn insert(&self, key: CacheKey, bytes: Vec<u8>, ttl: Duration) {
    if ttl.is_zero() {
      return;
    }

    let expires_at = Instant::now() + ttl;
    self.cache.insert(key, CachedResponse { expires_at, bytes });
  }
}
