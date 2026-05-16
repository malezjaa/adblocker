mod application;
mod blocker;
mod blocklists;
mod cache;
mod cert;
mod config;
mod db;
mod doh;
mod domain;
pub mod dot;
mod firewall;
mod logger;
mod server;
mod state;
mod windows;

use crate::application::app::App;
use crate::blocker::{lookup_block, BlockLookup};
use crate::blocklists::load_blocklists;
use crate::cert::get_certs;
use crate::doh::setup_doh_server;
use crate::dot::setup_dot_server;
use crate::logger::setup_logger;
use crate::server::setup_server;
use crate::state::State;
use adblock::Engine;
use anyhow::Result;
use chrono::Duration as ChronoDuration;
use fs_err::create_dir_all;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::task::LocalSet;
use tokio::time::sleep;
use tokio::{join, spawn};
use tracing::{error, info};

async fn retry_task<F, Fut>(name: &'static str, delay: Duration, f: F)
where
  F: Fn() -> Fut,
  Fut: Future<Output = Result<()>>,
{
  loop {
    if let Err(err) = f().await {
      error!(error = ?err, "{name} failed, restarting in {delay:?}");
      sleep(delay).await;
    }
  }
}

macro_rules! task {
  ($name:literal, $dur:expr, $future:block) => {
    futures::future::OptionFuture::from(Some(spawn(retry_task(
      $name,
      Duration::from_secs($dur),
      $future,
    ))))
  };
  ($name:literal, $dur:expr, $condition:expr, $future:block) => {
    futures::future::OptionFuture::from(if $condition {
      Some(spawn(retry_task(
        $name,
        Duration::from_secs($dur),
        $future,
      )))
    } else {
      None
    })
  };
}

#[tokio::main]
async fn main() -> Result<()> {
  setup_logger();
  let certs = get_certs()?;
  let home_path = dirs::home_dir().unwrap().join("adb");
  let cache_dir = home_path.join("cache");

  create_dir_all(&cache_dir)?;

  let db_path = home_path.join("dns-adblock.sqlite");
  let (tx, rx) = mpsc::channel::<BlockLookup>(100);
  let state = State::from_paths(home_path.join("config.toml"), db_path, tx).await?;

  let start = Instant::now();
  let rules = load_blocklists(state.clone(), &cache_dir).await?;

  info!("loaded lists in {:.2?}", start.elapsed());
  let engine = Engine::from_filter_set(rules, true);

  async fn run_engine(engine: Engine, mut rx: mpsc::Receiver<BlockLookup>) -> Result<()> {
    while let Some(lookup) = rx.recv().await {
      lookup.sender.send(lookup_block(&engine, &lookup.msg, lookup.origin)).ok();
    }
    Ok(())
  }

  let local = LocalSet::new();
  local.spawn_local(run_engine(engine, rx));

  let state = state.clone();
  local
    .run_until(async {
      let _cleanup = state.clone().spawn_cleanup_task(ChronoDuration::days(30));
      let backend = task!("Dashboard Backend", 3, state.config().dashboard_enabled(), {
        let s = state.clone();
        move || setup_server(s.clone())
      });

      let doh = task!("DoH server", 3, state.config().doh_enabled(), {
        let s = state.clone();
        move || setup_doh_server(s.clone())
      });

      let dot = task!("DoT server", 3, state.config().dot_enabled(), {
        let s = state.clone();
        let certs = certs.clone();
        move || setup_dot_server(s.clone(), certs.clone())
      });

      let dns = task!("DNS server", 3, {
        let s = state.clone();
        move || {
          let s = s.clone();
          async move { App::init(s).await?.run().await }
        }
      });

      let _ = join!(dns, backend, doh, dot);
    })
    .await;

  Ok(())
}
