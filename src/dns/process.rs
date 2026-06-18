use crate::context::Context;
use crate::dashboard::ws::WsEvent;
use crate::dns::resolve::resolve_msg;
use crate::engine::message::{BlockLookup, BlockOrigin, BlockResult};
use crate::engine::{EngineMessage, handle_blocked_response};
use crate::rewrite::apply::{apply_rewrites, restore_original_queries};
use hickory_proto::op::Message;
use hickory_proto::serialize::binary::BinDecodable;
use tokio::sync::oneshot;

pub async fn process_message(
  ctx: Context,
  raw: Vec<u8>,
  origin: BlockOrigin,
) -> anyhow::Result<(bool, Message)> {
  let mut msg = Message::from_bytes(&raw)?;
  let original_queries = msg.queries.clone();

  let rewrite_result = apply_rewrites(&ctx, &mut msg)?;

  let (tx, rx) = oneshot::channel();

  let _ = ctx.ws_tx().send(WsEvent::DNSRequest);
  ctx
    .tx()
    .send(EngineMessage::Lookup(BlockLookup::new(msg.clone(), tx).origin(origin)))
    .await?;

  match rx.await? {
    BlockResult::Block => Ok((true, handle_blocked_response(&msg)?)),

    BlockResult::Ok => {
      let mut response = if rewrite_result.synthetic_response {
        msg.clone().into_response()
      } else {
        resolve_msg(&msg, ctx).await?
      };

      if rewrite_result.restore_original_queries {
        restore_original_queries(&mut response, &original_queries);
      }

      Ok((false, response))
    }
  }
}
