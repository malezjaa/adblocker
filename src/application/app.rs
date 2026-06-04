use crate::blocklists::load_blocklists;
use crate::config::Config;
use crate::context::Context;
use crate::database::DB;
use crate::engine::BlockLookup;
use crate::firewall::external_dns::block_external_dns;
use crate::firewall::open_port::open_ports;
use crate::firewall::override_dns::override_default_dns;
use crate::run_engine;
use crate::task::named_task;
#[cfg(windows)]
use crate::windows::wfp_session::WfpSession;
use adblock::Engine;
use anyhow::Result;
use chrono::Duration;
use std::time::Instant;
use tokio::sync::mpsc::Receiver;
use tokio::task::{JoinSet, LocalSet};
use tracing::log::warn;
use tracing::{error, info};
use windows::Win32::Foundation::HANDLE;

#[derive(Clone)]
pub struct App {
  #[cfg(windows)]
  pub wfp_sess: WfpSession,
  pub ctx: Context,
}

impl App {
  #[cfg(windows)]
  pub async fn init(ctx: Context) -> Result<Self> {
    let engine = HANDLE::default();
    override_default_dns(ctx.socket(), ctx.secondary_name_server())?;
    open_ports(engine)?;

    Ok(Self { ctx, wfp_sess: WfpSession { engine } })
  }

  #[cfg(not(windows))]
  pub async fn init(ctx: Context) -> Result<Self> {
    override_default_dns(ctx.socket(), ctx.secondary_name_server())?;
    open_ports()?;

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
        Config::spawn_config_watcher(self.ctx.clone())?;

        let mut tasks = JoinSet::new();

        tasks.spawn_local(named_task("AdBlocking engine", run_engine(engine, rx)));
        tasks.spawn(named_task(
          "DB cleanup",
          DB::spawn_cleanup_task(self.ctx.db().pool.clone(), Duration::days(30)),
        ));

        tasks.spawn(named_task("DNS", Self::start_dns(self.ctx.clone())));
        if config.dot_enabled() {
          tasks.spawn(named_task("DoT", Self::start_dot(self.ctx.clone())));
        }
        if config.doh_enabled() {
          tasks.spawn(named_task("DoH", Self::start_doh(self.ctx.clone())));
        }
        if config.dashboard_enabled() {
          tasks.spawn(named_task("Dashboard", Self::start_dashboard(self.ctx.clone())));
        }
        // mutex guard would be held across await point below
        drop(config);

        while let Some(result) = tasks.join_next().await {
          match result {
            Ok(Ok(())) => {
              warn!("a background task exited unexpectedly");
            }
            Ok(Err(err)) => {
              error!("{:?}", err);
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
