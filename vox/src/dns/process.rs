use crate::context::Context;
use crate::dashboard::ws::WsEvent;
use crate::engine::EngineMessage;
use crate::engine::message::{BlockLookup, BlockResult};
use anyhow::{Result, bail};
use hickory_proto::op::{Message, ResponseCode, UpdateMessage};
use hickory_proto::rr::rdata::opt::{EdnsCode, EdnsOption};
use hickory_proto::rr::rdata::{A, AAAA};
use hickory_proto::rr::{RData, Record, RecordType};
use hickory_proto::serialize::binary::BinDecodable;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Instant;
use tokio::sync::oneshot;
use vox_dns::block_origin::BlockOrigin;
use vox_dns::edns::EDNSCode;
use vox_dns::rewrite::apply::{
  RewriteContext, apply_rewrites_with_context, restore_original_queries,
};

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

fn origin_from_edns(msg: &Message, fallback: BlockOrigin) -> Result<BlockOrigin> {
  let Some(edns) = &msg.edns else {
    return Ok(fallback);
  };

  let Some(EdnsOption::Unknown(_, data)) =
    edns.option(EdnsCode::Unknown(EDNSCode::BlockOrigin as u16))
  else {
    return Ok(fallback);
  };

  let Some(origin) = data.first() else {
    bail!("EDNS BlockOrigin option is empty");
  };

  BlockOrigin::from_u8(*origin)
}

impl Context {
  async fn process_message(
    &self,
    raw: Vec<u8>,
    mut origin: BlockOrigin,
    device: Option<&str>,
  ) -> Result<(bool, Message, BlockOrigin)> {
    let mut msg = Message::from_bytes(&raw)?;

    // Clients can overwrite some settings using EDNS.
    origin = origin_from_edns(&msg, origin)?;

    let original_queries = msg.queries.clone();

    let rewrite_result = {
      let config = self.config();
      apply_rewrites_with_context(
        config.rewrites.as_deref(),
        &mut msg,
        RewriteContext { origin: Some(origin), device },
      )?
    };

    let _ = self.ws_tx().send(WsEvent::DNSRequest);

    if rewrite_result.skip_block_lookup {
      let mut response = msg.clone().into_response();

      if rewrite_result.restore_original_queries {
        restore_original_queries(
          &mut response,
          &original_queries,
          &rewrite_result.rewritten_names,
        );
      }

      return Ok((false, response, origin));
    }

    let (tx, rx) = oneshot::channel();
    self
      .tx()
      .send(EngineMessage::Lookup(BlockLookup::new(msg.clone(), tx).origin(origin)))
      .await?;

    match rx.await? {
      BlockResult::Block => {
        let mut response = handle_blocked_response(&msg)?;

        if rewrite_result.restore_original_queries {
          restore_original_queries(
            &mut response,
            &original_queries,
            &rewrite_result.rewritten_names,
          );
        }

        Ok((true, response, origin))
      }

      BlockResult::Ok => {
        let mut response = if rewrite_result.synthetic_response {
          msg.clone().into_response()
        } else {
          self.resolve_msg(&msg).await?
        };

        if rewrite_result.restore_original_queries {
          restore_original_queries(
            &mut response,
            &original_queries,
            &rewrite_result.rewritten_names,
          );
        }

        Ok((false, response, origin))
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
    let (blocked, response, origin) =
      self.process_message(bytes, origin, device.as_deref()).await?;

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

#[cfg(test)]
mod tests {
  use super::*;
  use hickory_proto::op::{Edns, Query};
  use hickory_proto::rr::Name;
  use std::net::{Ipv4Addr, Ipv6Addr};
  use std::str::FromStr;
  use vox_dns::block_origin::{ClientOrigin, TransportOrigin};

  fn query(record_type: RecordType) -> Message {
    let mut msg = Message::query();
    msg.add_query(Query::query(Name::from_str("blocked.test.").unwrap(), record_type));
    msg
  }

  fn query_with_origin(origin: BlockOrigin) -> Message {
    let mut msg = query(RecordType::A);
    let mut edns = Edns::new();
    edns
      .options_mut()
      .insert(EdnsOption::Unknown(EDNSCode::BlockOrigin as u16, vec![origin.to_u8()]));
    msg.edns = Some(edns);
    msg
  }

  #[test]
  fn blocked_a_queries_return_loopback_answer() {
    let response = handle_blocked_response(&query(RecordType::A)).unwrap();

    assert_eq!(response.response_code, ResponseCode::NoError);
    assert_eq!(response.answers.len(), 1);
    assert_eq!(response.answers[0].ttl, 5);
    assert_eq!(response.answers[0].data, RData::A(A(Ipv4Addr::LOCALHOST)));
  }

  #[test]
  fn blocked_aaaa_queries_return_ipv6_loopback_answer() {
    let response = handle_blocked_response(&query(RecordType::AAAA)).unwrap();

    assert_eq!(response.response_code, ResponseCode::NoError);
    assert_eq!(response.answers.len(), 1);
    assert_eq!(response.answers[0].ttl, 5);
    assert_eq!(response.answers[0].data, RData::AAAA(AAAA(Ipv6Addr::LOCALHOST)));
  }

  #[test]
  fn blocked_non_address_queries_return_nxdomain() {
    let response = handle_blocked_response(&query(RecordType::TXT)).unwrap();

    assert_eq!(response.response_code, ResponseCode::NXDomain);
    assert!(response.answers.is_empty());
    assert_eq!(response.queries.len(), 1);
  }

  #[test]
  fn edns_block_origin_overrides_fallback_origin() {
    let expected = BlockOrigin::Client {
      client: ClientOrigin::Windows,
      transport: TransportOrigin::DoH,
    };
    let msg = query_with_origin(expected);

    assert_eq!(origin_from_edns(&msg, BlockOrigin::plain()).unwrap(), expected);
  }

  #[test]
  fn missing_edns_block_origin_keeps_fallback_origin() {
    let mut msg = query(RecordType::A);
    msg.edns = Some(Edns::new());

    assert_eq!(origin_from_edns(&msg, BlockOrigin::doh()).unwrap(), BlockOrigin::doh());
  }

  #[test]
  fn empty_edns_block_origin_returns_error_instead_of_panicking() {
    let mut msg = query(RecordType::A);
    let mut edns = Edns::new();
    edns
      .options_mut()
      .insert(EdnsOption::Unknown(EDNSCode::BlockOrigin as u16, Vec::new()));
    msg.edns = Some(edns);

    let err = origin_from_edns(&msg, BlockOrigin::plain()).unwrap_err();

    assert!(err.to_string().contains("BlockOrigin option is empty"));
  }
}
