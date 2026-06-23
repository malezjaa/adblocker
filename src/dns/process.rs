use crate::context::Context;
use crate::dashboard::ws::WsEvent;
use crate::engine::EngineMessage;
use crate::engine::message::{BlockLookup, BlockOrigin, BlockResult};
use crate::rewrite::apply::{apply_rewrites, restore_original_queries};
use anyhow::Result;
use hickory_proto::op::{Message, ResponseCode, UpdateMessage};
use hickory_proto::rr::rdata::{A, AAAA};
use hickory_proto::rr::{RData, Record, RecordType};
use hickory_proto::serialize::binary::BinDecodable;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Instant;
use tokio::sync::oneshot;

pub fn handle_blocked_response(msg: &Message) -> anyhow::Result<Message> {
  let mut response = Message::response(msg.id(), msg.op_code).into_response();
  response.add_queries(msg.queries.clone());

  for query in &msg.queries {
    let rdata = match query.query_type() {
      RecordType::A => Some(RData::A(A(Ipv4Addr::new(127, 0, 0, 1)))),
      RecordType::AAAA => Some(RData::AAAA(AAAA(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)))),
      _ => None,
    };

    if let Some(rdata) = rdata {
      let record = Record::from_rdata(query.name().clone(), 5, rdata);
      response.add_answer(record);
    }
  }

  if response.answers.is_empty() {
    response.metadata.response_code = ResponseCode::NXDomain;
  }

  Ok(response)
}

impl Context {
  async fn process_message(
    &self,
    raw: Vec<u8>,
    origin: BlockOrigin,
  ) -> anyhow::Result<(bool, Message)> {
    let mut msg = Message::from_bytes(&raw)?;
    let original_queries = msg.queries.clone();

    let rewrite_result = apply_rewrites(self, &mut msg)?;

    let (tx, rx) = oneshot::channel();

    let _ = self.ws_tx().send(WsEvent::DNSRequest);
    self
      .tx()
      .send(EngineMessage::Lookup(BlockLookup::new(msg.clone(), tx).origin(origin)))
      .await?;

    match rx.await? {
      BlockResult::Block => Ok((true, handle_blocked_response(&msg)?)),

      BlockResult::Ok => {
        let mut response = if rewrite_result.synthetic_response {
          msg.clone().into_response()
        } else {
          self.resolve_msg(&msg).await?
        };

        if rewrite_result.restore_original_queries {
          restore_original_queries(&mut response, &original_queries);
        }

        Ok((false, response))
      }
    }
  }

  pub async fn query_dns(
    &self,
    bytes: Vec<u8>,
    origin: BlockOrigin,
    addr: SocketAddr,
    device: Option<String>,
  ) -> Result<Message> {
    let start = Instant::now();
    let (blocked, response) = self.process_message(bytes, origin).await?;

    self
      .db()
      .record_query(
        &response,
        addr,
        blocked,
        origin,
        start.elapsed().as_millis() as i64,
        device,
      )
      .await;

    Ok(response)
  }
}
