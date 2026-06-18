use crate::context::Context;
use moka::sync::Cache;

#[derive(Clone)]
pub enum CacheEntry {
  Resolved(String),
  Blocked,
}

pub struct DnsCache(Cache<String, Option<CacheEntry>>);

impl DnsCache {
  pub fn new() -> Self {
    Self(Cache::new(10_000))
  }

  pub fn is_cached(&self, key: impl Into<String>) -> bool {
    self.0.contains_key(&key.into())
  }
}
