use crate::context::Context;
use crate::engine::EngineActor;
use crate::engine::cache::{CacheLookup, MAX_NEGATIVE_TTL};
use anyhow::bail;
use hickory_proto::op::{Message, Query, ResponseCode};
use hickory_proto::rr::{Name, RData, RecordType};
use hickory_resolver::lookup::Lookup;
use hickory_resolver::net::{DnsError, NetError, NoRecords};
use std::time::Duration;
use tracing::{debug, warn};

impl Context {
  pub async fn resolve_msg(&self, msg: &Message) -> anyhow::Result<Message> {
    let Some(query) = msg.queries.first() else { bail!("No name or record") };
    let query_name = query.name.to_owned();

    if let Some(response) = self.try_cache(msg, query) {
      return Ok(response);
    }

    let mut response = msg.clone().into_response();
    match self.resolver().lookup(query_name.clone(), query.query_type).await {
      Ok(lookup) => {
        self.handle_resolved(&query_name, query.query_type, lookup, &mut response)
      }
      Err(e) => {
        self.handle_resolve_error(&query_name, query.query_type, e, &mut response)?
      }
    }

    Ok(response)
  }

  fn try_cache(&self, msg: &Message, query: &Query) -> Option<Message> {
    match self.cache().get(query.name(), query.query_type()) {
      CacheLookup::Resolved(cached) => {
        debug!(name = %query.name(), record_type = ?query.query_type(), "dns cache hit");
        let mut response = msg.clone().into_response();
        response.metadata.response_code = cached.response_code;
        for record in cached.records {
          response.add_answer(record);
        }
        Some(response)
      }
      CacheLookup::Blocked => {
        warn!(name = %query.name(), "resolve_msg saw a Blocked cache entry...");
        None
      }
      CacheLookup::Miss => None,
    }
  }

  fn handle_resolved(
    &self,
    query_name: &Name,
    query_type: RecordType,
    lookup: Lookup,
    response: &mut Message,
  ) {
    let records = lookup.answers().to_vec();
    let ttl = lookup.answers().iter().map(|r| r.ttl).min().unwrap_or(60);

    self.cache().insert_resolved(
      query_name.clone(),
      query_type,
      records.clone(),
      ResponseCode::NoError,
      Duration::from_secs(ttl as u64),
    );

    for record in records {
      response.add_answer(record);
    }
  }

  fn handle_resolve_error(
    &self,
    query_name: &Name,
    query_type: RecordType,
    e: NetError,
    response: &mut Message,
  ) -> anyhow::Result<()> {
    match e {
      NetError::Dns(DnsError::NoRecordsFound(no)) => {
        response.metadata.response_code = no.response_code;
        let duration = negative_ttl(&no);
        debug!(name = %query_name, ttl = ?duration, "no records found");
        self.cache().insert_resolved(
          query_name.clone(),
          query_type,
          Vec::new(),
          no.response_code,
          duration,
        );
        Ok(())
      }
      NetError::Timeout => {
        warn!(name = %query_name, "upstream resolver timed out");
        response.metadata.response_code = ResponseCode::ServFail;
        Ok(())
      }
      _ => {
        warn!(error = ?e, name = %query_name, "unhandled resolver error kind");
        Err(e.into())
      }
    }
  }
}

fn negative_ttl(no: &NoRecords) -> Duration {
  let secs = no
    .authorities
    .as_ref()
    .map(|auths| {
      auths
        .iter()
        .filter_map(
          |r| if let RData::SOA(soa) = &r.data { Some(soa.minimum) } else { None },
        )
        .min()
        .unwrap_or(60)
    })
    .unwrap_or(60);
  Duration::from_secs(secs.min(MAX_NEGATIVE_TTL) as u64)
}
