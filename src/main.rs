mod application;
mod blocklists;
mod cache;
mod config;
mod context;
mod middleware;
mod middlewares;
mod firewall;
mod windows;

use std::time::Duration;
use crate::application::app::App;
use anyhow::Result;
use tokio::time::sleep;
use tracing::error;

fn setup_logger() {
  tracing_subscriber::fmt().with_env_filter("dns_adblock=info").init();
}

#[tokio::main]
async fn main() -> Result<()> {
  setup_logger();
  loop {
    if let Err(err) = App::init().await?.run().await {
      sleep(Duration::from_secs(3)).await;
      error!(error = ?err, "dns adblocker failed. trying to restart in 3s");
    }
  }
}
