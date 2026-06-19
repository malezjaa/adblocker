// engine/cache.rs

use hickory_proto::op::ResponseCode;
use hickory_proto::rr::{Name, Record, RecordType};
use moka::sync::Cache;
use std::time::{Duration, Instant};

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct CacheKey {
  pub name: Name,
  pub record_type: RecordType,
}

#[derive(Clone)]
pub struct ResolvedCacheEntry {
  pub records: Vec<Record>,
  pub response_code: ResponseCode,
  pub expires_at: Instant,
}

impl ResolvedCacheEntry {
  pub fn is_expired(&self) -> bool {
    Instant::now() >= self.expires_at
  }
}

#[derive(Clone)]
pub enum CacheEntry {
  Resolved(ResolvedCacheEntry),
  Blocked,
}

pub enum CacheLookup {
  Resolved(ResolvedCacheEntry),
  Blocked,
  Miss,
}

pub struct DnsCache(Cache<CacheKey, CacheEntry>);

impl DnsCache {
  pub fn new() -> Self {
    Self(Cache::builder().max_capacity(10_000).build())
  }

  pub fn is_cached(&self, name: &Name, record_type: RecordType) -> bool {
    self.0.contains_key(&CacheKey { name: name.clone(), record_type })
  }

  pub fn is_blocked(&self, name: &Name, record_type: RecordType) -> bool {
    self
      .0
      .get(&CacheKey { name: name.clone(), record_type })
      .is_some_and(|entry| matches!(entry, CacheEntry::Blocked))
  }

  pub fn insert_blocked(&self, name: Name, record_type: RecordType) {
    self.0.insert(CacheKey { name, record_type }, CacheEntry::Blocked);
  }

  pub fn insert_resolved(
    &self,
    name: Name,
    record_type: RecordType,
    records: Vec<Record>,
    response_code: ResponseCode,
    ttl: Duration,
  ) {
    self.0.insert(
      CacheKey { name, record_type },
      CacheEntry::Resolved(ResolvedCacheEntry {
        records,
        response_code,
        expires_at: Instant::now() + ttl,
      }),
    );
  }

  pub fn get(&self, name: &Name, record_type: RecordType) -> CacheLookup {
    let key = CacheKey { name: name.clone(), record_type };

    match self.0.get(&key) {
      Some(CacheEntry::Resolved(entry)) if !entry.is_expired() => {
        CacheLookup::Resolved(entry)
      }
      Some(CacheEntry::Resolved(_)) => {
        self.0.invalidate(&key);
        CacheLookup::Miss
      }
      Some(CacheEntry::Blocked) => CacheLookup::Blocked,
      None => CacheLookup::Miss,
    }
  }
}
