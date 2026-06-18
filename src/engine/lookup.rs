use crate::engine::EngineActor;
use crate::engine::cache::DnsCache;
use crate::engine::message::{BlockOrigin, BlockResult};
use adblock::Engine;
use adblock::request::Request;
use hickory_proto::op::Message;
use tracing::info;

impl EngineActor {
  pub fn lookup_block(&self, msg: &Message, origin: BlockOrigin) -> BlockResult {
    for query in &msg.queries {
      let host = query.name().to_string();
      let host = host.trim_end_matches('.');
      if self.cache.contains_key(host) {
        info!(?host, ?origin, "blocked from cache");
        return BlockResult::Block;
      }

      let url = format!("https://{}/", host);
      if let Ok(req) = Request::new(&url, "", "document") {
        let res = self.engine.check_network_request(&req);
        if res.matched && res.exception.is_none() {
          info!(?host, ?origin, "blocked");
          self.cache.insert(host.to_owned(), ());
          return BlockResult::Block;
        }
      }
    }

    BlockResult::Ok
  }
}
