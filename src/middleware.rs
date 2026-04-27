use hickory_proto::op::Message;
use async_trait::async_trait;
use crate::context::Context;

#[allow(dead_code)]
pub enum MiddlewareResult {
  /// Pass to next middleware
  Next,
  /// Return NXDOMAIN, stop chain
  Block,
  /// Return custom response, stop chain
  Respond(Message),
}

#[async_trait(?Send)]
pub trait Middleware {
  async fn process(&self, ctx: &mut Context) -> MiddlewareResult;
}
