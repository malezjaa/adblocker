// context.rs resolve_msg

use crate::context::Context;
use crate::engine::EngineActor;
use crate::engine::cache::{CacheLookup, MAX_NEGATIVE_TTL};
use anyhow::bail;
use hickory_proto::op::{Message, ResponseCode};
use hickory_proto::rr::{RData, RecordType};
use hickory_resolver::net::{DnsError, NetError};
use std::time::Duration;
use tracing::{debug, warn};

impl Context {
  pub async fn resolve_msg(&self, msg: &Message) -> anyhow::Result<Message> {
    let Some(query) = msg.queries.first() else { bail!("No name or record") };
    let query_name = query.name.to_owned();

    match self.cache().get(query.name(), query.query_type()) {
      CacheLookup::Resolved(cached) => {
        debug!(
            name = %query.name(),
            record_type = ?query.query_type(),
            "dns cache hit"
        );

        let mut response = msg.clone().into_response();
        response.metadata.response_code = cached.response_code;

        for record in cached.records {
          response.add_answer(record);
        }

        return Ok(response);
      }
      CacheLookup::Blocked => {
        warn!(
            name = %query.name(),
            "resolve_msg saw a Blocked cache entry. block checks should happen before resolution"
        );
      }
      CacheLookup::Miss => {}
    }

    let mut response = msg.clone().into_response();
    match self.resolver().lookup(query_name.clone(), query.query_type).await {
      Ok(lookup) => {
        let records = lookup.answers().to_vec();

        let ttl = lookup.answers().iter().map(|r| r.ttl).min().unwrap_or(60);

        self.cache().insert_resolved(
          query_name.clone(),
          query.query_type,
          records.clone(),
          ResponseCode::NoError,
          Duration::from_secs(ttl as u64),
        );

        for record in records {
          response.add_answer(record);
        }
      }
      Err(e) => match e {
        NetError::Dns(DnsError::NoRecordsFound(no)) => {
          response.metadata.response_code = no.response_code;
          let duration = if let Some(authorities) = no.authorities {
            let secs = authorities
              .iter()
              .filter_map(
                |r| if let RData::SOA(soa) = &r.data { Some(soa) } else { None },
              )
              .map(|soa| soa.minimum)
              .min()
              .unwrap_or(60);
            Duration::from_secs(secs.min(MAX_NEGATIVE_TTL) as u64)
          } else {
            Duration::from_secs(60)
          };

          debug!(name = %query_name, ttl = ?duration, "no records found");
          self.cache().insert_resolved(
            query_name.clone(),
            query.query_type,
            Vec::new(),
            no.response_code,
            duration,
          );
        }
        NetError::Timeout => {
          warn!(name = %query_name, "upstream resolver timed out");
          response.metadata.response_code = ResponseCode::ServFail;
        }
        _ => {
          warn!(error = ?e, name = %query_name, "unhandled resolver error kind");
          return Err(e.into());
        }
      },
    }

    Ok(response)
  }
}
