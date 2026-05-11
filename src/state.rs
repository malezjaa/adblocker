use crate::blocker::BlockLookup;
use crate::config::Config;
use crate::server::ws::WsEvent;
use anyhow::Result;
use hickory_resolver::config::{CLOUDFLARE, GOOGLE, ResolverConfig};
use hickory_resolver::{TokioResolver, net::runtime::TokioRuntimeProvider};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::{RwLock, broadcast};

#[derive(Debug, Clone)]
pub struct State(pub Arc<StateImpl>);

#[derive(Debug)]
pub struct StateImpl {
  pub tx: Sender<BlockLookup>,
  pub ws_tx: broadcast::Sender<WsEvent>,
  pub config: RwLock<Config>,
  pub db: SqlitePool,
  pub total_queries: AtomicUsize,
  pub resolver: TokioResolver,
}

impl State {
  pub fn new(config: Config, db: SqlitePool, tx: Sender<BlockLookup>) -> Result<Self> {
    let mut r_config = ResolverConfig::udp_and_tcp(&CLOUDFLARE);
    for ns in GOOGLE.udp_and_tcp() {
      r_config.add_name_server(ns);
    }

    let mut resolver_builder =
      TokioResolver::builder_with_config(r_config, TokioRuntimeProvider::default());

    let opts = resolver_builder.options_mut();
    opts.negative_min_ttl = Some(Duration::from_secs(60));
    opts.positive_min_ttl = Some(Duration::from_secs(60));

    Ok(Self(Arc::new(StateImpl {
      tx,
      config: RwLock::new(config),
      db,
      total_queries: AtomicUsize::default(),
      resolver: resolver_builder.build()?,
      ws_tx: broadcast::channel(100).0,
    })))
  }

  pub fn tx(&self) -> Sender<BlockLookup> {
    self.0.tx.clone()
  }

  pub fn resolver(&self) -> &TokioResolver {
    &self.0.resolver
  }

  pub async fn blocklists(&self) -> Vec<String> {
    self.0.config.read().await.blocklists.clone()
  }

  pub async fn socket(&self) -> SocketAddr {
    self.0.config.read().await.socket
  }

  pub async fn secondary_name_server(&self) -> Option<SocketAddr> {
    self.0.config.read().await.secondary_name_server
  }

  pub fn ws_tx(&self) -> broadcast::Sender<WsEvent> {
    self.0.ws_tx.clone()
  }
}
