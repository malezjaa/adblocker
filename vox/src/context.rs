use crate::certs::Certs;
use crate::dashboard::ws::WsEvent;
use crate::database::DB;
use crate::dns::resolver::create_hickory_resolver;
use crate::engine::EngineMessage;
use crate::mmdb::downloader::{MMDBSPaths, download_mmdbs_files};
use crate::mmdb::mmdbs::MMDBS;
use anyhow::Result;
use fs_err::create_dir_all;
use hickory_resolver::TokioResolver;
use parking_lot::{RwLock, RwLockReadGuard};
use rustls::ServerConfig;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use tracing::log::trace;
use vox_dns::cache::DnsCache;
use vox_shared::config::Config;
use vox_shared::home_dir;

#[derive(Clone)]
pub struct Context(pub Arc<ContextImpl>);

pub struct ContextImpl {
  pub tx: Sender<EngineMessage>,
  pub ws_tx: broadcast::Sender<WsEvent>,
  pub config: RwLock<Config>,
  pub resolver: RwLock<TokioResolver>,
  pub db: DB,
  pub cache_dir: PathBuf,
  pub server_config: Arc<ServerConfig>,
  pub config_path: PathBuf,
  pub mmdbs: RwLock<Option<MMDBS>>,
  pub paths: MMDBSPaths,
  pub dns_cache: DnsCache,
  pub rules_version: AtomicU64,
}

impl Context {
  pub async fn new(tx: Sender<EngineMessage>) -> Result<Self> {
    let home_path = home_dir();
    let cache_dir = home_path.join("cache");

    create_dir_all(&cache_dir)?;

    let db_path = home_path.join("vox.sqlite");
    let db = DB::init(db_path).await?;

    let config_path = home_path.join("config.toml");
    let config = Config::from_file(&config_path)?;

    let certs = Certs::load_certs()?;
    let mut server_config = ServerConfig::builder()
      .with_no_client_auth()
      .with_single_cert(certs.certs, certs.key)?;
    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    let paths = download_mmdbs_files();

    let resolver = create_hickory_resolver(&config)?;
    let ctx = Self(Arc::new(ContextImpl {
      tx,
      config: RwLock::new(config),
      db,
      resolver: RwLock::new(resolver),
      ws_tx: broadcast::channel(100).0,
      cache_dir,
      server_config: Arc::new(server_config),
      config_path,
      mmdbs: RwLock::new(None),
      paths,
      dns_cache: DnsCache::new(),
      rules_version: AtomicU64::new(0),
    }));

    ctx.db().attach_context(&ctx);

    Ok(ctx)
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

  pub fn tx(&self) -> Sender<EngineMessage> {
    self.0.tx.clone()
  }

  pub fn blocklists(&self) -> Vec<String> {
    self.0.config.read().blocklists.clone()
  }

  pub fn socket(&self) -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], self.config().dns.port))
  }

  pub fn doh_socket(&self) -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], self.config().doh.port))
  }

  pub fn config(&self) -> RwLockReadGuard<'_, Config> {
    self.0.config.read()
  }

  pub fn ws_tx(&self) -> broadcast::Sender<WsEvent> {
    self.0.ws_tx.clone()
  }

  pub fn config_path(&self) -> &Path {
    self.0.config_path.as_ref()
  }

  pub fn cache(&self) -> &DnsCache {
    &self.0.dns_cache
  }

  pub fn resolver(&self) -> TokioResolver {
    self.0.resolver.read().clone()
  }

  pub fn update_resolver(&self, new_resolver: TokioResolver) {
    *self.0.resolver.write() = new_resolver;
    trace!("updated hickory resolver");
  }

  pub fn update_config(&self, config: Config) {
    *self.0.config.write() = config;
    trace!("updated in-memory config");
  }

  pub fn increment_rules_version(&self) {
    self.0.rules_version.fetch_add(1, Ordering::Relaxed);
  }

  pub fn rules_version(&self) -> u64 {
    self.0.rules_version.load(Ordering::Relaxed)
  }
}
