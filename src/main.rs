mod application;
mod blocklists;
mod cache;
mod cert;
mod config;
mod context;
mod dashboard;
mod db;
mod domain;
mod engine;
mod firewall;
mod logger;
pub mod mmdb;
mod rewrite;
pub mod task;
mod windows;

use crate::application::app::App;
use crate::context::Context;
use crate::engine::{BlockLookup, lookup_block};
use crate::logger::setup_logger;
use adblock::Engine;
use anyhow::Result;
use chrono::Duration as ChronoDuration;
use clap::Parser;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::channel;

async fn run_engine(engine: Engine, mut rx: mpsc::Receiver<BlockLookup>) -> Result<()> {
  while let Some(lookup) = rx.recv().await {
    lookup.sender.send(lookup_block(&engine, &lookup.msg, lookup.origin)).ok();
  }
  Ok(())
}

#[derive(Parser, Debug)]
struct Cli {
  #[arg(short, long)]
  verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
  let cli = Cli::parse();
  setup_logger(cli.verbose);

  let (tx, rx) = channel(100);
  let ctx = Context::new(tx).await?;
  ctx.load_mmdbs()?;

  let app = Arc::new(App::init(ctx).await?);
  app.start_all(rx).await?;

  Ok(())
}
