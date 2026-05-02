use crate::context::Context;
use async_trait::async_trait;
use hickory_proto::op::Message;

#[allow(dead_code)]
pub enum MiddlewareResult {
  Next,
  Block,
  Respond(Message),
}

#[async_trait(?Send)]
pub trait Middleware {
  async fn process(&self, ctx: &mut Context) -> MiddlewareResult;
}
