mod application;
mod blocklists;
mod cache;
mod cert;
mod config;
mod context;
mod dashboard;
mod domain;
mod engine;
mod firewall;
mod logger;
pub mod mmdb;
mod rewrite;
pub mod task;
mod windows;
pub mod database;

use crate::application::app::App;
use crate::context::Context;
use crate::engine::{lookup_block, BlockLookup};
use crate::logger::setup_logger;
use adblock::Engine;
use anyhow::Result;
use chrono::Duration as ChronoDuration;
use clap::Parser;
use rustls::crypto::ring;
use scopeguard::defer;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::channel;
use ::windows::Win32::NetworkManagement::WindowsFilteringPlatform::FwpmEngineClose0;

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
  ring::default_provider()
    .install_default()
    .expect("failed to install rustls crypto provider");
  let cli = Cli::parse();
  setup_logger(cli.verbose);

  let (tx, rx) = channel(100);
  let ctx = Context::new(tx).await?;
  ctx.load_mmdbs()?;

  let app = Arc::new(App::init(ctx).await?);
  defer! {
    #[cfg(windows)] unsafe {
      FwpmEngineClose0(app.wfp_sess.engine);
    }
  }
  app.start_all(rx).await?;

  Ok(())
}
