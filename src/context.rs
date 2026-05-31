use crate::cert::{Certs, get_certs};
use crate::config::Config;
use crate::dashboard::ws::WsEvent;
use crate::db::DB;
use crate::engine::BlockLookup;
use crate::mmdb::downloader::download_mmdbs_files;
use anyhow::Result;
use fs_err::create_dir_all;
use hickory_resolver::config::{CLOUDFLARE, GOOGLE, ResolverConfig};
use hickory_resolver::{TokioResolver, net::runtime::TokioRuntimeProvider};
use parking_lot::{RwLock, RwLockReadGuard};
use rustls::ServerConfig;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
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
  pub resolver: TokioResolver,
  pub db: DB,
  pub cache_dir: PathBuf,
  pub server_config: Arc<ServerConfig>,
  pub config_path: PathBuf,
}

impl Context {
  pub async fn new(tx: Sender<BlockLookup>) -> Result<Self> {
    let Certs { key, certs } = get_certs()?;
    let home_path = dirs::home_dir().unwrap().join("adb");
    let cache_dir = home_path.join("cache");

    create_dir_all(&cache_dir)?;

    let db_path = home_path.join("dns-adblock.sqlite");
    let db = DB::from_path(db_path).await?;

    let config_path = home_path.join("config.toml");
    let mut config = Config::from_file(&config_path)?;
    config.compile_regexes()?;

    let mut r_config = ResolverConfig::udp_and_tcp(&CLOUDFLARE);
    for ns in GOOGLE.udp_and_tcp() {
      r_config.add_name_server(ns);
    }

    let mut resolver_builder =
      TokioResolver::builder_with_config(r_config, TokioRuntimeProvider::default());

    let opts = resolver_builder.options_mut();
    opts.negative_min_ttl = Some(Duration::from_secs(60));
    opts.positive_min_ttl = Some(Duration::from_secs(60));

    let server_config = Arc::new(
      ServerConfig::builder().with_no_client_auth().with_single_cert(certs, key)?,
    );

    download_mmdbs_files().await?;

    Ok(Self(Arc::new(ContextImpl {
      tx,
      config: RwLock::new(config),
      db,
      resolver: resolver_builder.build()?,
      ws_tx: broadcast::channel(100).0,
      cache_dir,
      server_config,
      config_path,
    })))
  }

  pub fn server_config(&self) -> Arc<ServerConfig> {
    self.0.server_config.clone()
  }

  pub fn db(&self) -> &DB {
    &self.0.db
  }

  pub fn cache_dir(&self) -> &Path {
    self.0.cache_dir.as_ref()
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
    SocketAddr::from(([0, 0, 0, 0], 53))
  }

  pub fn config(&self) -> RwLockReadGuard<'_, Config> {
    self.0.config.read()
  }

  pub fn secondary_name_server(&self) -> Option<SocketAddr> {
    self.0.config.read().secondary_name_server
  }

  pub fn ws_tx(&self) -> broadcast::Sender<WsEvent> {
    self.0.ws_tx.clone()
  }

  pub fn config_path(&self) -> &Path {
    self.0.config_path.as_ref()
  }
}
