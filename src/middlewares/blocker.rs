use crate::context::Context;
use crate::middleware::{Middleware, MiddlewareResult};
use adblock::request::Request;
use adblock::{Engine, FilterSet};
use async_trait::async_trait;
use std::sync::Arc;
use hickory_proto::rr::RecordType;
use tokio::sync::Mutex;
use tracing::info;

pub struct Blocker {
  pub engine: Arc<Mutex<Engine>>,
}

impl Blocker {
  pub fn new(filters: FilterSet) -> Self {
    Self {
      engine: Arc::new(Mutex::new(Engine::from_filter_set(filters, true))),
    }
  }
}

#[async_trait(?Send)]
impl Middleware for Blocker {
  async fn process(&self, ctx: &mut Context) -> MiddlewareResult {
    let engine = self.engine.lock().await;

    for query in &ctx.msg().queries {
      let host = query.name().to_string();
      let host = host.trim_end_matches('.');
      let url = format!("http://{}/", host);

      let request_type = match query.query_type() {
        RecordType::A => "script",
        _ => "other",
      };

      if let Ok(req) = Request::new(&url, "http://example.com", request_type) {
        let res = engine.check_network_request(&req);
        if res.matched && res.exception.is_none() {
          info!(?url, "blocked");
          return MiddlewareResult::Block;
        }
      }
    }

    MiddlewareResult::Next
  }
}
