mod application;
mod blocker;
mod blocklists;
mod cache;
mod cert;
mod config;
mod db;
mod domain;
mod firewall;
mod logger;
mod server;
mod context;
mod windows;
pub mod task;

use crate::application::app::App;
use crate::blocker::{lookup_block, BlockLookup};
use crate::blocklists::load_blocklists;
use crate::cert::get_certs;
use crate::context::Context;
use crate::logger::setup_logger;
use adblock::Engine;
use anyhow::Result;
use chrono::Duration as ChronoDuration;
use fs_err::create_dir_all;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::task::LocalSet;
use tokio::time::sleep;
use tokio::{join, spawn};
use tracing::{error, info};


#[tokio::main]
async fn main() -> Result<()> {
  setup_logger();
  let certs = get_certs()?;
  let home_path = dirs::home_dir().unwrap().join("adb");
  let cache_dir = home_path.join("cache");

  create_dir_all(&cache_dir)?;

  let db_path = home_path.join("dns-adblock.sqlite");
  let (tx, rx) = mpsc::channel::<BlockLookup>(100);
  let ctx = Context::from_paths(home_path.join("config.toml"), db_path, tx, certs).await?;

  let start = Instant::now();
  let rules = load_blocklists(ctx.clone(), &cache_dir).await?;

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

  let app = Arc::new(App::init(ctx).await?);
  app.start_all().await?;

  Ok(())
}
