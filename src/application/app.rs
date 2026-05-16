use crate::context::Context;
use crate::firewall::external_dns::block_external_dns;
use crate::firewall::override_dns::override_default_dns;
use crate::task::spawn_task;
use anyhow::{bail, Result};
use hickory_proto::op::Message;
use hickory_resolver::net::{DnsError, NetError};

#[derive(Clone)]
pub struct App {
  pub ctx: Context,
}

impl App {
  pub async fn init(ctx: Context) -> Result<Self> {
    override_default_dns(ctx.socket(), ctx.secondary_name_server())?;
    block_external_dns(ctx.socket())?;

    Ok(Self { ctx })
  }

  pub async fn start_all(&self) -> Result<()> {
    let config = self.ctx.config();

    let tasks = vec![
      spawn_task("DNS server", true, Self::start_dns(self.ctx.clone())),
      spawn_task("DoT server", config.dot_enabled(), Self::start_dot(self.ctx.clone())),
      spawn_task("DoH server", config.doh_enabled(), Self::start_doh(self.ctx.clone())),
      spawn_task("Dashboard backend", config.dashboard_enabled(), Self::start_dashboard(self.ctx.clone()))
    ];

    for task in tasks {
      if let Some(task) = task {
        task.await?
      }
    }

    Ok(())
  }
}

pub async fn resolve_msg(msg: &Message, ctx: Context) -> Result<Message> {
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
      _ => return Err(e.into()),
    },
  }

  Ok(response)
}
