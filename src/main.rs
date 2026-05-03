mod application;
mod blocklists;
mod cache;
mod config;
mod blocker;
mod firewall;
mod windows;

use crate::application::app::App;
use crate::blocker::{lookup_block, BlockLookup};
use crate::blocklists::load_blocklists;
use crate::config::Config;
use adblock::Engine;
use anyhow::Result;
use axum::routing::get;
use axum::Router;
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

async fn root() -> &'static str {
  "Hello, World!"
}

#[tokio::main]
async fn main() -> Result<()> {
  setup_logger();
  let home_path = dirs::home_dir().unwrap().join("adb");
  let config = Config::from_file(home_path.join("config.toml"))?;
  let cache_dir = home_path.join("cache");

  if !cache_dir.exists() {
    create_dir_all(&cache_dir)?;
  }

  let start = Instant::now();
  let rules = load_blocklists(config.blocklists, &cache_dir).await?;
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

  local.run_until(async {
    let dns = spawn(async move {
      loop {
        if let Err(err) = App::init(config.socket, tx.clone()).await?.run().await {
          sleep(Duration::from_secs(3)).await;
          error!(error = ?err, "dns adblocker failed. trying to restart in 3s");
        }
      }

      Ok::<(), anyhow::Error>(())
    });

    let server = spawn(async move {
      let app = Router::new()
        .route("/", get(root));

      let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
      axum::serve(listener, app).await.unwrap();
    });

    let _ = join!(dns, server);
  }).await;

  Ok(())
}
