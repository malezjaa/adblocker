use crate::context::Context;
use anyhow::bail;
use hickory_proto::op::Message;
use hickory_resolver::net::{DnsError, NetError};
use tracing::warn;

pub async fn resolve_msg(msg: &Message, ctx: Context) -> anyhow::Result<Message> {
  let Some(query) = msg.queries.first() else { bail!("No name or record") };

  let mut response = msg.clone().into_response();

  match ctx.resolver().lookup(query.name.to_owned(), query.query_type).await {
    Ok(lookup) => {
      for record in lookup.answers() {
        response.add_answer(record.clone());
      }
    }
    Err(e) => match e {
      NetError::Dns(DnsError::NoRecordsFound(no)) => {
        response.metadata.response_code = no.response_code;
      }
      NetError::Timeout => {
        warn!("upstream resolver timed out");
      }
      _ => return Err(e.into()),
    },
  }

  Ok(response)
}
