mod application;
mod blocker;
mod blocklists;
mod cache;
mod config;
mod firewall;
mod server;
mod state;
mod windows;

use crate::application::app::App;
use crate::blocker::{lookup_block, BlockLookup};
use crate::blocklists::load_blocklists;
use crate::server::setup_server;
use crate::state::State;
use adblock::Engine;
use anyhow::Result;
use axum::routing::get;
use axum::Router;
use chrono::Duration as ChronoDuration;
use fs_err::create_dir_all;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tokio::task::LocalSet;
use tokio::time::sleep;
use tokio::{join, spawn};
use tracing::{error, info};

fn setup_logger() {
  tracing_subscriber::fmt().with_env_filter("dns_adblock=info").init();
}

#[tokio::main]
async fn main() -> Result<()> {
  setup_logger();
  let home_path = dirs::home_dir().unwrap().join("adb");
  let cache_dir = home_path.join("cache");

  create_dir_all(&cache_dir)?;

  let db_path = home_path.join("dns-adblock.sqlite");
  let state = State::from_paths(home_path.join("config.toml"), db_path).await?;

  let blocklists = state.blocklists().await;
  let socket = state.socket().await;
  let start = Instant::now();
  let rules = load_blocklists(blocklists, &cache_dir).await?;

  info!("loaded lists in {:.2?}", start.elapsed());
  let engine = Engine::from_filter_set(rules, true);

  let (tx, rx) = mpsc::channel::<BlockLookup>(100);

  async fn run_engine(engine: Engine, mut rx: mpsc::Receiver<BlockLookup>) -> Result<()> {
    while let Some(lookup) = rx.recv().await {
      lookup.sender.send(lookup_block(&engine, &lookup.msg)).ok();
    }
    Ok(())
  }

  let local = LocalSet::new();
  local.spawn_local(run_engine(engine, rx));

  local
    .run_until(async {
      let cleanup_state = state.clone();
      let dns = spawn(async move {
        loop {
          if let Err(err) =
            App::init(socket, tx.clone(), state.clone()).await?.run().await
          {
            sleep(Duration::from_secs(3)).await;
            error!(error = ?err, "dns adblocker failed. trying to restart in 3s");
          }
        }

        Ok::<(), anyhow::Error>(())
      });

      let server = spawn(setup_server());

      let _cleanup = cleanup_state.spawn_cleanup_task(ChronoDuration::days(30));
      let _ = join!(dns, server);
    })
    .await;

  Ok(())
}
