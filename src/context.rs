use anyhow::Result;
use hickory_proto::op::UpdateMessage;
use hickory_proto::op::{Message, Query, ResponseCode};
use hickory_proto::rr::rdata::{A, AAAA};
use hickory_proto::rr::{RData, Record, RecordType};
use hickory_proto::serialize::binary::BinDecodable;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::net::UdpSocket;

#[derive(Debug)]
pub struct Context {
  msg: Message,
}

impl Context {
  pub fn new(raw: Vec<u8>) -> Result<Self> {
    Ok(Self { msg: Message::from_bytes(&raw)? })
  }

  pub fn msg(&self) -> &Message {
    &self.msg
  }

  pub async fn send_blocked(&self, socket: &UdpSocket, src: SocketAddr) -> Result<()> {
    let mut response = Message::response(self.msg.id(), self.msg.op_code);

    response.add_queries(self.msg.queries.clone());

    for query in &self.msg.queries {
      let rdata = match query.query_type() {
        RecordType::A => Some(RData::A(A(Ipv4Addr::new(0, 0, 0, 0)))),
        RecordType::AAAA => {
          Some(RData::AAAA(AAAA(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0))))
        }
        _ => None,
      };

      if let Some(rdata) = rdata {
        let record = Record::from_rdata(query.name().clone(), 0, rdata);
        response.add_answer(record);
      }
    }

    if response.answers.is_empty() {
      response.metadata.response_code = ResponseCode::NXDomain;
    }

    let bytes = response.to_vec()?;
    socket.send_to(&bytes, src).await?;
    Ok(())
  }

  pub fn query(&self) -> Option<&Query> {
    self.msg().queries.first()
  }
}
