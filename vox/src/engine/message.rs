use hickory_proto::op::Message;
use tokio::sync::oneshot::Sender;
use vox_dns::block_origin::BlockOrigin;

#[derive(Debug)]
pub enum BlockResult {
  Ok,
  Block,
}

pub struct BlockLookup {
  pub msg: Message,
  pub sender: Sender<BlockResult>,
  pub origin: BlockOrigin,
}

impl BlockLookup {
  pub fn new(msg: Message, sender: Sender<BlockResult>) -> Self {
    Self { msg, sender, origin: BlockOrigin::plain() }
  }

  pub fn origin(mut self, origin: BlockOrigin) -> Self {
    self.origin = origin;
    self
  }
}
