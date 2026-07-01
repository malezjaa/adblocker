pub struct EngineActor {
  engine: Engine,
  pub ctx: Context,
}

use crate::context::Context;
pub(crate) use crate::engine::message::BlockLookup;
use crate::lists::downloader::load_blocklists;
use adblock::Engine;
use anyhow::Result;
use tokio::sync::mpsc;

pub mod lookup;
pub mod message;

pub enum EngineMessage {
  Lookup(BlockLookup),
  ReloadFilterSet,
}

impl EngineActor {
  pub async fn new(context: Context) -> Result<Self> {
    let rules = load_blocklists(&context).await?;
    let engine = Engine::from_filter_set(rules, true);

    Ok(Self { ctx: context, engine })
  }

  pub async fn run(&mut self, mut rx: mpsc::Receiver<EngineMessage>) -> Result<()> {
    while let Some(message) = rx.recv().await {
      match message {
        EngineMessage::Lookup(lookup) => {
          let result = self.ctx.lookup_block(&self.engine, &lookup.msg, lookup.origin);

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
