use anyhow::{Result, anyhow};
use futures::StreamExt;
use hickory_client::{
  client::Client,
  proto::{
    DnsHandle,
    op::{Edns, Message, MessageType, OpCode, Query},
    rr::rdata::opt::EdnsOption,
    xfer::{DnsRequest, DnsRequestOptions, DnsResponse},
  },
};

use crate::edns::EDNSCode;
#[derive(Debug)]
pub struct DnsQuery {
  msg: Message,
}

impl DnsQuery {
  pub fn from_query(query: Query) -> Self {
    let mut msg = Message::new();

    msg.set_id(rand::random());
    msg.set_message_type(MessageType::Query);
    msg.set_op_code(OpCode::Query);
    msg.set_recursion_desired(true);
    msg.add_query(query);

    let mut edns = Edns::new();
    edns.set_max_payload(1234);
    msg.set_edns(edns);

    Self { msg }
  }

  pub fn from_message(mut msg: Message) -> Self {
    let mut edns = Edns::new();
    edns.set_max_payload(1234);
    msg.set_edns(edns);

    Self { msg }
  }

  pub fn add_edns_option(mut self, code: EDNSCode, data: &[u8]) -> Self {
    if let Some(edns) = self.msg.extensions_mut() {
      let opt = EdnsOption::Unknown(code as u16, data.to_vec());
      edns.options_mut().insert(opt);
    }

    self
  }

  pub async fn send(self, client: &Client) -> Result<DnsResponse> {
    let mut options = DnsRequestOptions::default();
    options.use_edns = true;

    let request = DnsRequest::new(self.msg, options);

    let mut responses = client.send(request);

    let response = responses
      .next()
      .await
      .ok_or_else(|| anyhow!("DNS server returned no response"))??;

    Ok(response)
  }
}
