use crate::application::app::resolve_msg;
use crate::state::State;
use adblock::Engine;
use adblock::request::Request;
use anyhow::Result;
use hickory_proto::op::{Message, ResponseCode, UpdateMessage};
use hickory_proto::rr::rdata::{A, AAAA};
use hickory_proto::rr::{RData, Record, RecordType};
use hickory_proto::serialize::binary::BinDecodable;
use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;
use tracing::info;

pub struct BlockLookup {
  pub msg: Message,
  pub sender: Sender<BlockResult>,
  pub doh: bool,
}

impl BlockLookup {
  pub fn new(msg: Message, sender: Sender<BlockResult>) -> Self {
    Self { msg, sender, doh: false }
  }

  pub fn doh(mut self) -> Self {
    self.doh = true;
    self
  }
}

#[derive(Debug)]
pub enum BlockResult {
  Ok,
  Block,
}

pub fn lookup_block(engine: &Engine, msg: &Message, doh: bool) -> BlockResult {
  for query in &msg.queries {
    let host = query.name().to_string();
    let host = host.trim_end_matches('.');
    let url = format!("https://{}/", host);

    if let Ok(req) = Request::new(&url, "", "document") {
      let res = engine.check_network_request(&req);
      if res.matched && res.exception.is_none() {
        info!(?url, ?doh, "blocked");
        return BlockResult::Block;
      }
    }
  }

  BlockResult::Ok
}

pub async fn check_block(
  state: State,
  raw: Vec<u8>,
  doh: bool,
) -> Result<(bool, Message)> {
  let msg = Message::from_bytes(&raw)?;
  let (sender, rx) = oneshot::channel();

  let mut lookup = BlockLookup::new(msg.clone(), sender);
  lookup = if doh { lookup.doh() } else { lookup };

  state.tx().send(lookup).await?;

  Ok(match rx.await? {
    BlockResult::Ok => (false, resolve_msg(&msg, state.clone()).await?),
    BlockResult::Block => (true, handle_blocked_response(&msg)?),
  })
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
