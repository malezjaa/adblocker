use crate::context::Context;
use dashmap::DashMap;
use hickory_proto::op::ResponseCode;
use hickory_proto::rr::{Name, Record, RecordType};
use moka::sync::Cache;
use std::time::{Duration, Instant};
use tokio::sync::broadcast::Sender;
use tracing::log::trace;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct CacheKey {
  pub name: Name,
  pub record_type: RecordType,
}

#[derive(Clone, Debug)]
pub struct ResolvedCacheEntry {
  pub records: Vec<Record>,
  pub response_code: ResponseCode,
  pub expires_at: Instant,
  pub rules_version: u64,
}

impl ResolvedCacheEntry {
  pub fn is_expired(&self) -> bool {
    Instant::now() >= self.expires_at
  }
}

#[derive(Clone, Debug)]
pub enum CacheEntry {
  Resolved(ResolvedCacheEntry),
  Blocked(u64),
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
      trace!("in flight guard dropped");
      let _ = tx.send(());
    }
  }
}

#[derive(Debug, Clone)]
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

  pub fn is_blocked(&self, key: &CacheKey, current_rules_version: u64) -> bool {
    self.cache.get(key).is_some_and(|entry| match entry {
      CacheEntry::Blocked(rules_version) => if rules_version == current_rules_version {
        true
      }  else {
        self.cache.invalidate(key);
        false
      }
      _ => false
    })
  }

  pub fn insert_blocked(&self, key: CacheKey, version: u64) {
    self.cache.insert(key, CacheEntry::Blocked(version));
  }

  pub fn insert_resolved(
    &self,
    key: CacheKey,
    records: Vec<Record>,
    response_code: ResponseCode,
    ttl: Duration,
    rules_version: u64,
  ) {
    self.cache.insert(
      key,
      CacheEntry::Resolved(ResolvedCacheEntry {
        records,
        response_code,
        expires_at: Instant::now() + ttl,
        rules_version,
      }),
    );
  }

  pub fn get(&self, key: &CacheKey, current_rules_version: u64) -> CacheLookup {
    match self.cache.get(key) {
      Some(CacheEntry::Resolved(entry)) => {
        if !entry.is_expired() && entry.rules_version == current_rules_version {
          CacheLookup::Resolved(entry)
        } else {
          self.cache.invalidate(key);
          CacheLookup::Miss
        }
      }
      Some(CacheEntry::Blocked(rules_version)) => if rules_version == current_rules_version { CacheLookup::Blocked } else {
        self.cache.invalidate(key);
        CacheLookup::Miss
      },
      None => CacheLookup::Miss,
    }
  }

  pub fn in_flight(&self) -> &InFlight {
    &self.in_flight
  }
}
