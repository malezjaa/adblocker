use std::time::{Duration, Instant};

use anyhow::{anyhow, bail};
use dashmap::Entry;
use hickory_proto::op::{Message, ResponseCode};
use hickory_resolver::{
  lookup::Lookup,
  net::{DnsError, NetError},
};
use tokio::sync::broadcast;
use tracing::{trace, warn};
use vox_dns::{
  cache::{CacheKey, CacheLookup, InFlightGuard},
  ttl::negative_ttl,
};

use crate::context::Context;

impl Context {
  pub async fn resolve_msg(&self, msg: &Message) -> anyhow::Result<Message> {
    let Some(query) = msg.queries.first() else { bail!("No name or record") };
    let query_name = query.name.to_owned();
    let cache_key = CacheKey { name: query_name, record_type: query.query_type };

    if let Some(response) = self.try_cache(msg, &cache_key) {
      return Ok(response);
    }

    let mut rx_follower = None;
    let mut _leader_guard = None;

    match self.cache().in_flight().entry(cache_key.clone()) {
      Entry::Occupied(occ) => {
        rx_follower = Some(occ.get().subscribe());
        trace!(name = %cache_key.name, record_type = ?cache_key.record_type, "listening to existing in-flight leader");
      }
      Entry::Vacant(vac) => {
        let (tx, _) = broadcast::channel(100);
        vac.insert(tx);
        _leader_guard =
          Some(InFlightGuard { cache: self.cache(), key: cache_key.clone() });
        trace!(name = %cache_key.name, record_type = ?cache_key.record_type, "new in-flight leader created");
      }
    }
    let mut response = msg.clone().into_response();

    if let Some(mut rx) = rx_follower {
      rx.recv().await.map_err(|_| anyhow!("in-flight lookup dropped"))?;
      if let Some(response) = self.try_cache(msg, &cache_key) {
        return Ok(response);
      }
    }

    let start = Instant::now();
    match self.resolver().lookup(cache_key.name.clone(), query.query_type).await {
      Ok(lookup) => {
        trace!(name = %cache_key.name, record_type = ?cache_key.record_type, "upstream took {:.2?} to resolve", start.elapsed());
        self.handle_resolved(cache_key, lookup, &mut response)
      }
      Err(e) => self.handle_resolve_error(cache_key, e, &mut response)?,
    }

    Ok(response)
  }

  fn try_cache(&self, msg: &Message, key: &CacheKey) -> Option<Message> {
    match self.cache().get(key, self.rules_version()) {
      CacheLookup::Resolved(cached) => {
        let mut response = msg.clone().into_response();
        response.metadata.response_code = cached.response_code;
        for record in cached.records {
          response.add_answer(record);
        }
        Some(response)
      }
      CacheLookup::Blocked => {
        warn!(name = %key.name, "resolve_msg saw a Blocked cache entry...");
        None
      }
      CacheLookup::Miss => None,
    }
  }

  fn handle_resolved(&self, cache_key: CacheKey, lookup: Lookup, response: &mut Message) {
    let records = lookup.answers().to_vec();
    let ttl = lookup.answers().iter().map(|r| r.ttl).min().unwrap_or(60);

    self.cache().insert_resolved(
      cache_key,
      records.clone(),
      ResponseCode::NoError,
      Duration::from_secs(ttl as u64),
      self.rules_version(),
    );

    for record in records {
      response.add_answer(record);
    }
  }

  fn handle_resolve_error(
    &self,
    cache_key: CacheKey,
    e: NetError,
    response: &mut Message,
  ) -> anyhow::Result<()> {
    match e {
      NetError::Dns(DnsError::NoRecordsFound(no)) => {
        response.metadata.response_code = no.response_code;
        let duration = negative_ttl(&no);
        trace!(name = %cache_key.name, ttl = ?duration, "no records found");
        self.cache().insert_resolved(
          cache_key,
          Vec::new(),
          no.response_code,
          duration,
          self.rules_version(),
        );
        Ok(())
      }
      NetError::Timeout => {
        warn!(name = %cache_key.name, "upstream resolver timed out");
        response.metadata.response_code = ResponseCode::ServFail;
        Ok(())
      }
      _ => {
        warn!(error = ?e, name = %cache_key.name, "unhandled resolver error kind");
        Err(e.into())
      }
    }
  }
}
