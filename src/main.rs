mod blocklists;
mod cache;
mod config;
mod context;
mod middleware;
mod middlewares;
mod response_cache;
mod app;

use crate::app::app::App;
use crate::blocklists::load_blocklists;
use crate::config::Config;
use crate::context::Context;
use crate::middleware::MiddlewareResult;
use crate::response_cache::ResponseCache;
use anyhow::Result;
use app::pipeline::Pipeline;
use fs_err::create_dir_all;
use hickory_proto::op::{Message, ResponseCode, UpdateMessage};
use hickory_proto::serialize::binary::{BinDecodable, BinEncodable};
use middlewares::blocker::Blocker;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tracing::{debug, error, info};

fn setup_logger() {
  tracing_subscriber::fmt()
    .with_env_filter("dns_adblock=info")
    .init();
}

#[tokio::main]
async fn main() -> Result<()> {
  setup_logger();
  loop {
    if let Err(err) = App::init().await?.run().await {
      error!(error = ?err, "dns adblocker failed. trying to restart");
    }
  }
}

