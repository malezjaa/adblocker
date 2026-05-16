use crate::blocker::BlockLookup;
use crate::cert::Certs;
use crate::config::Config;
use crate::server::ws::WsEvent;
use anyhow::Result;
use hickory_resolver::config::{ResolverConfig, CLOUDFLARE, GOOGLE};
use hickory_resolver::{net::runtime::TokioRuntimeProvider, TokioResolver};
use parking_lot::{RwLock, RwLockReadGuard};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct Context(pub Arc<ContextImpl>);

#[derive(Debug)]
pub struct ContextImpl {
  pub tx: Sender<BlockLookup>,
  pub ws_tx: broadcast::Sender<WsEvent>,
  pub config: RwLock<Config>,
  pub db: SqlitePool,
  pub total_queries: AtomicUsize,
  pub resolver: TokioResolver,
  pub certs: Certs,
}

impl Context {
  pub fn new(config: Config, db: SqlitePool, tx: Sender<BlockLookup>, certs: Certs) -> Result<Self> {
    let mut r_config = ResolverConfig::udp_and_tcp(&CLOUDFLARE);
    for ns in GOOGLE.udp_and_tcp() {
      r_config.add_name_server(ns);
    }

    let mut resolver_builder =
      TokioResolver::builder_with_config(r_config, TokioRuntimeProvider::default());

    let opts = resolver_builder.options_mut();
    opts.negative_min_ttl = Some(Duration::from_secs(60));
    opts.positive_min_ttl = Some(Duration::from_secs(60));

    Ok(Self(Arc::new(ContextImpl {
      tx,
      config: RwLock::new(config),
      db,
      total_queries: AtomicUsize::default(),
      resolver: resolver_builder.build()?,
      ws_tx: broadcast::channel(100).0,
      certs,
    })))
  }

  pub fn tx(&self) -> Sender<BlockLookup> {
    self.0.tx.clone()
  }

  pub fn resolver(&self) -> &TokioResolver {
    &self.0.resolver
  }

  pub fn blocklists(&self) -> Vec<String> {
    self.0.config.read().blocklists.clone()
  }

  pub fn block_rules(&self) -> Option<Vec<String>> {
    self.0.config.read().block_rules.clone()
  }

  pub fn socket(&self) -> SocketAddr {
    self.0.config.read().socket
  }

  pub fn config(&self) -> RwLockReadGuard<Config> {
    self.0.config.read()
  }

  pub fn secondary_name_server(&self) -> Option<SocketAddr> {
    self.0.config.read().secondary_name_server
  }

  pub fn ws_tx(&self) -> broadcast::Sender<WsEvent> {
    self.0.ws_tx.clone()
  }
  
  pub fn certs(&self) -> &Certs {
    &self.0.certs
  }
}
