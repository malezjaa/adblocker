use crate::context::Context;
use crate::engine::EngineActor;
use crate::engine::cache::{CacheKey, DnsCache};
use crate::engine::message::{BlockOrigin, BlockResult};
use adblock::Engine;
use adblock::request::Request;
use hickory_proto::op::Message;
use tracing::info;

impl Context {
  pub fn lookup_block(
    &self,
    engine: &Engine,
    msg: &Message,
    origin: BlockOrigin,
  ) -> BlockResult {
    for query in &msg.queries {
      let key = CacheKey { name: query.name.clone(), record_type: query.query_type };
      let host = query.name().to_string();
      let host = host.trim_end_matches('.');
      if self.cache().is_blocked(&key) {
        info!(?host, ?origin, "blocked from cache");
        return BlockResult::Block;
      }

      let url = format!("https://{}/", host);
      if let Ok(req) = Request::new(&url, "", "document") {
        let res = engine.check_network_request(&req);
        if res.matched && res.exception.is_none() {
          info!(?host, ?origin, "blocked");
          self.cache().insert_blocked(key);
          return BlockResult::Block;
        }
      }
    }

    BlockResult::Ok
  }
}
