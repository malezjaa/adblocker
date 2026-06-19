// engine/cache.rs

use dashmap::DashMap;
use hickory_proto::op::ResponseCode;
use hickory_proto::rr::{Name, Record, RecordType};
use moka::sync::Cache;
use std::time::{Duration, Instant};
use tokio::sync::broadcast::Sender;

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

#[derive(Clone)]
pub enum CacheLookup {
  Resolved(ResolvedCacheEntry),
  Blocked,
  Miss,
}

pub const MAX_NEGATIVE_TTL: u32 = 3600;

pub type InFlight = DashMap<CacheKey, Sender<()>>;
pub struct InFlightGuard<'a> {
  pub cache: &'a DnsCache,
  pub key: CacheKey,
}

impl Drop for InFlightGuard<'_> {
  fn drop(&mut self) {
    if let Some((_, tx)) = self.cache.in_flight().remove(&self.key) {
      let _ = tx.send(());
    }
  }
}

pub struct DnsCache {
  cache: Cache<CacheKey, CacheEntry>,
  in_flight: InFlight,
}

impl DnsCache {
  pub fn new() -> Self {
    Self {
      cache: Cache::builder().max_capacity(10_000).build(),
      in_flight: DashMap::new(),
    }
  }

  pub fn is_cached(&self, key: &CacheKey) -> bool {
    self.cache.contains_key(key)
  }

  pub fn is_blocked(&self, key: &CacheKey) -> bool {
    self.cache.get(key).is_some_and(|entry| matches!(entry, CacheEntry::Blocked))
  }

  pub fn insert_blocked(&self, key: CacheKey) {
    self.cache.insert(key, CacheEntry::Blocked);
  }

  pub fn insert_resolved(
    &self,
    key: CacheKey,
    records: Vec<Record>,
    response_code: ResponseCode,
    ttl: Duration,
  ) {
    self.cache.insert(
      key,
      CacheEntry::Resolved(ResolvedCacheEntry {
        records,
        response_code,
        expires_at: Instant::now() + ttl,
      }),
    );
  }

  pub fn get(&self, key: &CacheKey) -> CacheLookup {
    match self.cache.get(key) {
      Some(CacheEntry::Resolved(entry)) if !entry.is_expired() => {
        CacheLookup::Resolved(entry)
      }
      Some(CacheEntry::Resolved(_)) => {
        self.cache.invalidate(key);
        CacheLookup::Miss
      }
      Some(CacheEntry::Blocked) => CacheLookup::Blocked,
      None => CacheLookup::Miss,
    }
  }

  pub fn in_flight(&self) -> &InFlight {
    &self.in_flight
  }
}
