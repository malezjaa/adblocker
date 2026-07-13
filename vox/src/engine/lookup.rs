use adblock::{Engine, request::Request};
use hickory_proto::op::Message;
use tracing::info;
use vox_dns::{block_origin::BlockOrigin, cache::CacheKey};

use crate::{context::Context, engine::message::BlockResult};

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
      if self.cache().is_blocked(&key, self.rules_version()) {
        info!(?host, ?origin, "blocked from cache");
        return BlockResult::Block;
      }

      let url = format!("https://{}/", host);
      if let Ok(req) = Request::new(&url, "", "document", "") {
        let res = engine.check_network_request(&req);
        if res.should_block() {
          info!(?host, ?origin, "blocked");
          self.cache().insert_blocked(key, self.rules_version());
          return BlockResult::Block;
        }
      }
    }

    BlockResult::Ok
  }
}
