pub struct EngineActor {
  engine: Engine,
  cache: DnsCache,
  ctx: Context,
}

use crate::context::Context;
use crate::engine::cache::DnsCache;
pub(crate) use crate::engine::message::BlockLookup;
use crate::lists::downloader::load_blocklists;
use adblock::Engine;
use anyhow::Result;
use hickory_proto::op::{Message, ResponseCode, UpdateMessage};
use hickory_proto::rr::rdata::{A, AAAA};
use hickory_proto::rr::{RData, Record, RecordType};
use hickory_proto::serialize::binary::BinDecodable;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::sync::mpsc;

mod cache;
pub mod lookup;
pub mod message;

pub fn handle_blocked_response(msg: &Message) -> Result<Message> {
  let mut response = Message::response(msg.id(), msg.op_code).into_response();
  response.add_queries(msg.queries.clone());

  for query in &msg.queries {
    let rdata = match query.query_type() {
      RecordType::A => Some(RData::A(A(Ipv4Addr::new(127, 0, 0, 1)))),
      RecordType::AAAA => Some(RData::AAAA(AAAA(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)))),
      _ => None,
    };

    if let Some(rdata) = rdata {
      let record = Record::from_rdata(query.name().clone(), 300, rdata);
      response.add_answer(record);
    }
  }

  if response.answers.is_empty() {
    response.metadata.response_code = ResponseCode::NXDomain;
  }

  Ok(response)
}

pub enum EngineMessage {
  Lookup(BlockLookup),
  ReloadFilterSet,
}

impl EngineActor {
  pub async fn new(context: Context) -> Result<Self> {
    let rules = load_blocklists(&context).await?;
    let engine = Engine::from_filter_set(rules, true);

    Ok(Self { ctx: context, cache: DnsCache::new(), engine })
  }

  pub async fn run(&mut self, mut rx: mpsc::Receiver<EngineMessage>) -> Result<()> {
    while let Some(message) = rx.recv().await {
      match message {
        EngineMessage::Lookup(lookup) => {
          let result = self.lookup_block(&lookup.msg, lookup.origin);

          // we send result back using oneshot channel from block lookup
          lookup.sender.send(result).ok();
        }
        EngineMessage::ReloadFilterSet => {
          let rules = load_blocklists(&self.ctx).await?;
          self.engine = Engine::from_filter_set(rules, true);
        }
      }
    }
    Ok(())
  }
}
