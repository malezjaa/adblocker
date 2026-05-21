use crate::blocklists::load_blocklists;
use crate::context::Context;
use crate::db::DB;
use crate::engine::BlockLookup;
use crate::firewall::external_dns::block_external_dns;
use crate::firewall::override_dns::override_default_dns;
use crate::run_engine;
use adblock::Engine;
use anyhow::{Result, bail};
use chrono::Duration;
use hickory_proto::op::Message;
use hickory_resolver::net::{DnsError, NetError};
use std::time::Instant;
use tokio::sync::mpsc::Receiver;
use tokio::task::{JoinSet, LocalSet};
use tracing::log::warn;
use tracing::{error, info};

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

  pub async fn start_all(&self, rx: Receiver<BlockLookup>) -> Result<()> {
    let start = Instant::now();
    let rules = load_blocklists(self.ctx.clone()).await?;
    info!("loaded lists in {:.2?}", start.elapsed());

    let engine = Engine::from_filter_set(rules, true);

    let local = LocalSet::new();

    local
      .run_until(async {
        let config = self.ctx.config();
        println!("{:#?}", config);

        let mut tasks = JoinSet::new();

        tasks.spawn_local(run_engine(engine, rx));
        tasks
          .spawn(DB::spawn_cleanup_task(self.ctx.db().pool.clone(), Duration::days(30)));
        tasks.spawn(Self::start_dns(self.ctx.clone()));
        if config.dot_enabled() {
          tasks.spawn(Self::start_dot(self.ctx.clone()));
        }
        if config.doh_enabled() {
          tasks.spawn(Self::start_doh(self.ctx.clone()));
        }
        if config.dashboard_enabled() {
          tasks.spawn(Self::start_dashboard(self.ctx.clone()));
        }
        // mutex guard would be held across await point below
        drop(config);

        while let Some(result) = tasks.join_next().await {
          match result {
            Ok(Ok(())) => {
              warn!("a background task exited unexpectedly");
            }
            Ok(Err(err)) => {
              error!("a background task failed: {:?}", err);
            }
            Err(err) => {
              if err.is_cancelled() {
                warn!("a background task was cancelled");
              } else if err.is_panic() {
                error!("a background task panicked: {:?}", err);
              } else {
                error!("task join error: {:?}", err);
              }
            }
          }
        }

        Ok(())
      })
      .await
  }
}
