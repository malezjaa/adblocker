use crate::context::Context;
use crate::rewrite::apply::{apply_rewrites, restore_original_queries};
use crate::rewrite::record::construct_rewrite_records;
use crate::rewrite::{Rewrite, RewriteAction, RewriteMatchWhenType};
use adblock::Engine;
use adblock::request::Request;
use anyhow::{Error, Result, bail};
use hickory_proto::op::{Message, Query, ResponseCode, UpdateMessage};
use hickory_proto::rr::rdata::{A, AAAA};
use hickory_proto::rr::{Name, RData, Record, RecordType};
use hickory_proto::serialize::binary::BinDecodable;
use hickory_resolver::net::{DnsError, NetError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::vec;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;
use tracing::{info, warn};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum BlockOrigin {
  Plain,
  DoH,
  DoT,
}

pub struct BlockLookup {
  pub msg: Message,
  pub sender: Sender<BlockResult>,
  pub origin: BlockOrigin,
}

impl BlockLookup {
  pub fn new(msg: Message, sender: Sender<BlockResult>) -> Self {
    Self { msg, sender, origin: BlockOrigin::Plain }
  }

  pub fn origin(mut self, origin: BlockOrigin) -> Self {
    self.origin = origin;
    self
  }
}

#[derive(Debug)]
pub enum BlockResult {
  Ok,
  Block,
}

pub fn lookup_block(engine: &Engine, msg: &Message, origin: BlockOrigin) -> BlockResult {
  for query in &msg.queries {
    let host = query.name().to_string();
    let host = host.trim_end_matches('.');
    let url = format!("https://{}/", host);

    if let Ok(req) = Request::new(&url, "", "document") {
      let res = engine.check_network_request(&req);
      if res.matched && res.exception.is_none() {
        info!(?url, ?origin, "blocked");
        return BlockResult::Block;
      }
    }
  }

  BlockResult::Ok
}

pub async fn process_message(
  ctx: Context,
  raw: Vec<u8>,
  origin: BlockOrigin,
) -> Result<(bool, Message)> {
  let mut msg = Message::from_bytes(&raw)?;
  let original_queries = msg.queries.clone();

  let rewrite_result = apply_rewrites(&ctx, &mut msg)?;

  let (tx, rx) = oneshot::channel();

  ctx.tx().send(BlockLookup::new(msg.clone(), tx).origin(origin)).await?;

  match rx.await? {
    BlockResult::Block => Ok((true, handle_blocked_response(&msg)?)),

    BlockResult::Ok => {
      let mut response = if rewrite_result.synthetic_response {
        msg.clone().into_response()
      } else {
        resolve_msg(&msg, ctx).await?
      };

      if rewrite_result.restore_original_queries {
        restore_original_queries(&mut response, &original_queries);
      }

      Ok((false, response))
    }
  }
}

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

pub async fn resolve_msg(msg: &Message, ctx: Context) -> Result<Message> {
  let Some(query) = msg.queries.first() else { bail!("No name or record") };

  let mut response = msg.clone().into_response();

  match ctx.resolver().lookup(query.name.to_owned(), query.query_type).await {
    Ok(lookup) => {
      for record in lookup.answers() {
        response.add_answer(record.clone());
      }
    }
    Err(e) => match e {
      NetError::Dns(DnsError::NoRecordsFound(no)) => {
        response.metadata.response_code = no.response_code;
      }
      _ => return Err(e.into()),
    },
  }

  Ok(response)
}
