use std::time::{Duration, Instant};

use dashmap::DashMap;
use hickory_proto::{
  op::ResponseCode,
  rr::{Name, Record, RecordType},
};
use moka::sync::Cache;
use tokio::sync::broadcast::Sender;
use tracing::trace;

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

impl Default for DnsCache {
  fn default() -> Self {
    Self::new()
  }
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
      CacheEntry::Blocked(rules_version) => {
        if rules_version == current_rules_version {
          true
        } else {
          self.cache.invalidate(key);
          false
        }
      }
      _ => false,
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
      Some(CacheEntry::Blocked(rules_version)) => {
        if rules_version == current_rules_version {
          CacheLookup::Blocked
        } else {
          self.cache.invalidate(key);
          CacheLookup::Miss
        }
      }
      None => CacheLookup::Miss,
    }
  }

  pub fn in_flight(&self) -> &InFlight {
    &self.in_flight
  }
}

#[cfg(test)]
mod tests {
  use std::{net::Ipv4Addr, str::FromStr};

  use hickory_proto::rr::{RData, RecordType, rdata::A};

  use super::*;

  fn key(name: &str) -> CacheKey {
    CacheKey { name: Name::from_str(name).unwrap(), record_type: RecordType::A }
  }

  fn record(name: &str, ip: Ipv4Addr) -> Record {
    Record::from_rdata(Name::from_str(name).unwrap(), 60, RData::A(A(ip)))
  }

  #[test]
  fn resolved_entries_are_returned_until_rules_version_changes() {
    let cache = DnsCache::new();
    let key = key("example.test.");
    let records = vec![record("example.test.", Ipv4Addr::new(10, 0, 0, 2))];

    cache.insert_resolved(
      key.clone(),
      records.clone(),
      ResponseCode::NoError,
      Duration::from_secs(60),
      7,
    );

    match cache.get(&key, 7) {
      CacheLookup::Resolved(entry) => {
        assert_eq!(entry.records, records);
        assert_eq!(entry.response_code, ResponseCode::NoError);
      }
      CacheLookup::Blocked | CacheLookup::Miss => panic!("expected resolved cache hit"),
    }

    assert!(matches!(cache.get(&key, 8), CacheLookup::Miss));
    assert!(matches!(cache.get(&key, 7), CacheLookup::Miss));
  }

  #[test]
  fn expired_resolved_entries_are_invalidated() {
    let cache = DnsCache::new();
    let key = key("expired.test.");

    cache.insert_resolved(
      key.clone(),
      Vec::new(),
      ResponseCode::NoError,
      Duration::from_secs(0),
      1,
    );

    assert!(matches!(cache.get(&key, 1), CacheLookup::Miss));
  }

  #[test]
  fn blocked_entries_are_scoped_to_rules_version() {
    let cache = DnsCache::new();
    let key = key("blocked.test.");

    cache.insert_blocked(key.clone(), 3);

    assert!(cache.is_blocked(&key, 3));
    assert!(matches!(cache.get(&key, 3), CacheLookup::Blocked));
    assert!(!cache.is_blocked(&key, 4));
    assert!(matches!(cache.get(&key, 3), CacheLookup::Miss));
  }

  #[test]
  fn in_flight_guard_removes_entry_and_notifies_waiters() {
    let cache = DnsCache::new();
    let key = key("in-flight.test.");
    let (tx, mut rx) = tokio::sync::broadcast::channel(1);
    cache.in_flight().insert(key.clone(), tx);

    {
      let _guard = InFlightGuard { cache: &cache, key: key.clone() };
    }

    assert!(!cache.in_flight().contains_key(&key));
    assert!(rx.try_recv().is_ok());
  }
}
