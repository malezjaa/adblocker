use std::{
  net::SocketAddr,
  path::{Path, PathBuf},
  sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicU64, Ordering},
  },
};

use anyhow::Result;
use fs_err::create_dir_all;
use hickory_resolver::TokioResolver;
use parking_lot::{RwLock, RwLockReadGuard};
use rustls::ServerConfig;
use tokio::sync::{broadcast, mpsc::Sender};
use tracing::log::trace;
use vox_dns::cache::DnsCache;
use vox_shared::{
  config::{Config, certs::CertificateStrategy},
  home_dir,
};

use crate::{
  certs::Certs,
  dashboard::ws::WsEvent,
  database::DB,
  dns::resolver::create_hickory_resolver,
  engine::EngineMessage,
  mmdb::{
    downloader::{MMDBSPaths, download_mmdbs_files},
    mmdbs::MMDBS,
  },
};

#[derive(Clone)]
pub struct Context {
  inner: Arc<ContextImpl>,
}

pub struct ContextImpl {
  pub engine_channel: Sender<EngineMessage>,
  pub ws_tx: broadcast::Sender<WsEvent>,
  pub config: RwLock<Config>,
  pub resolver: RwLock<TokioResolver>,
  pub db: DB,
  pub cache_dir: PathBuf,
  pub server_config: Option<Arc<ServerConfig>>,
  pub config_path: PathBuf,
  pub mmdbs: RwLock<Option<MMDBS>>,
  pub paths: MMDBSPaths,
  pub dns_cache: DnsCache,
  pub rules_version: AtomicU64,
  pub blocklist_refresh_started: AtomicBool,
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

    let resolver = create_hickory_resolver(&config)?;
    let server_config = if matches!(config.certs.strategy, CertificateStrategy::None) {
      None
    } else {
      let certs = Certs::load_certs(&config, &resolver).await?;
      let mut server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs.certs, certs.key)?;
      server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

      Some(Arc::new(server_config))
    };

    let paths = download_mmdbs_files();

    let ctx = Self {
      inner: Arc::new(ContextImpl {
        engine_channel: tx,
        config: RwLock::new(config),
        db,
        resolver: RwLock::new(resolver),
        ws_tx: broadcast::channel(100).0,
        cache_dir,
        server_config,
        config_path,
        mmdbs: RwLock::new(None),
        paths,
        dns_cache: DnsCache::new(),
        rules_version: AtomicU64::new(0),
        blocklist_refresh_started: AtomicBool::new(false),
      }),
    };

    ctx.db().attach_context(&ctx);

    Ok(ctx)
  }

  pub fn db(&self) -> &DB {
    &self.inner.db
  }

  pub fn cache_dir(&self) -> &Path {
    self.inner.cache_dir.as_ref()
  }

  pub fn engine_channel(&self) -> Sender<EngineMessage> {
    self.inner.engine_channel.clone()
  }

  pub fn blocklists(&self) -> Vec<String> {
    self.inner.config.read().blocklists.clone()
  }

  pub fn socket(&self) -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], self.config().dns.port))
  }

  pub fn doh_socket(&self) -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], self.config().doh.port))
  }

  pub fn config(&self) -> RwLockReadGuard<'_, Config> {
    self.inner.config.read()
  }

  pub fn ws_tx(&self) -> broadcast::Sender<WsEvent> {
    self.inner.ws_tx.clone()
  }

  pub fn config_path(&self) -> &Path {
    self.inner.config_path.as_ref()
  }

  pub fn cache(&self) -> &DnsCache {
    &self.inner.dns_cache
  }

  pub fn resolver(&self) -> TokioResolver {
    self.inner.resolver.read().clone()
  }

  pub fn update_resolver(&self, new_resolver: TokioResolver) {
    *self.inner.resolver.write() = new_resolver;
    trace!("updated hickory resolver");
  }

  pub fn update_config(&self, config: Config) {
    *self.inner.config.write() = config;
    trace!("updated in-memory config");
  }

  pub fn increment_rules_version(&self) {
    self.inner.rules_version.fetch_add(1, Ordering::Relaxed);
  }

  pub fn rules_version(&self) -> u64 {
    self.inner.rules_version.load(Ordering::Relaxed)
  }

  pub fn start_blocklist_refresh(&self) -> bool {
    self
      .inner
      .blocklist_refresh_started
      .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
      .is_ok()
  }

  pub fn server_config(&self) -> Option<Arc<ServerConfig>> {
    self.inner.server_config.clone()
  }

  pub(crate) fn mmdb_paths(&self) -> MMDBSPaths {
    self.inner.paths.clone()
  }

  pub(crate) fn mmdbs(&self) -> RwLockReadGuard<'_, Option<MMDBS>> {
    self.inner.mmdbs.read()
  }

  pub(crate) fn replace_mmdbs(&self, mmdbs: MMDBS) {
    *self.inner.mmdbs.write() = Some(mmdbs);
  }

  pub(crate) fn downgrade(&self) -> Weak<ContextImpl> {
    Arc::downgrade(&self.inner)
  }

  pub(crate) fn from_weak(context: &Weak<ContextImpl>) -> Option<Self> {
    context.upgrade().map(|inner| Self { inner })
  }
}
