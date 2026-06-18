use hickory_proto::op::Message;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub enum BlockResult {
  Ok,
  Block,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum BlockOrigin {
  Plain,
  DoH,
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
