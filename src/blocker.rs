use crate::application::app::App;
use adblock::request::Request;
use adblock::Engine;
use anyhow::{anyhow, Result};
use hickory_proto::op::Message;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::sync::{mpsc, oneshot};
use tracing::info;

pub struct BlockLookup {
  pub msg: Message,
  pub sender: Sender<BlockResult>,
}

impl BlockLookup {
  pub fn new(msg: Message, sender: Sender<BlockResult>) -> Self {
    Self { msg, sender }
  }
}

pub enum BlockResult {
  Ok,
  Block,
}

pub fn lookup_block(engine: &Engine, msg: &Message) -> BlockResult {
  for query in &msg.queries {
    let host = query.name().to_string();
    let host = host.trim_end_matches('.');
    let url = format!("https://{}/", host);

    if let Ok(req) = Request::new(&url, "", "document") {
      let res = engine.check_network_request(&req);
      if res.matched && res.exception.is_none() {
        info!(?url, "blocked");
        return BlockResult::Block;
      }
    }
  }

  BlockResult::Ok
}
