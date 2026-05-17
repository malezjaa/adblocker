mod application;
mod blocker;
mod blocklists;
mod cache;
mod cert;
mod config;
mod context;
mod db;
mod domain;
mod firewall;
mod logger;
mod server;
pub mod task;
mod windows;

use crate::application::app::App;
use crate::blocker::{BlockLookup, lookup_block};
use crate::context::Context;
use crate::logger::setup_logger;
use adblock::Engine;
use anyhow::Result;
use chrono::Duration as ChronoDuration;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::channel;

async fn run_engine(engine: Engine, mut rx: mpsc::Receiver<BlockLookup>) -> Result<()> {
  while let Some(lookup) = rx.recv().await {
    lookup.sender.send(lookup_block(&engine, &lookup.msg, lookup.origin)).ok();
  }
  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
  setup_logger();

  let (tx, rx) = channel(100);
  let ctx = Context::new(tx).await?;

  let app = Arc::new(App::init(ctx).await?);
  app.start_all(rx).await?;

  Ok(())
}
