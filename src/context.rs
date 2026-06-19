use crate::cert::{Certs, get_certs};
use crate::config::Config;
use crate::dashboard::ws::WsEvent;
use crate::database::DB;
use crate::engine::EngineMessage;
use crate::engine::cache::DnsCache;
use crate::mmdb::downloader::{MMDBSPaths, download_mmdbs_files};
use crate::mmdb::mmdbs::MMDBS;
use anyhow::Result;
use fs_err::create_dir_all;
use hickory_resolver::config::{NameServerConfig, ResolverConfig};
use hickory_resolver::{TokioResolver, net::runtime::TokioRuntimeProvider};
use parking_lot::{RwLock, RwLockReadGuard};
use rustls::ServerConfig;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct Context(pub Arc<ContextImpl>);

pub struct ContextImpl {
  pub tx: Sender<EngineMessage>,
  pub ws_tx: broadcast::Sender<WsEvent>,
  pub config: RwLock<Config>,
  pub resolver: TokioResolver,
  pub db: DB,
  pub cache_dir: PathBuf,
  pub server_config: Arc<ServerConfig>,
  pub config_path: PathBuf,
  pub mmdbs: RwLock<Option<MMDBS>>,
  pub paths: MMDBSPaths,
  pub dns_cache: DnsCache,
}

impl Context {
  pub async fn new(tx: Sender<EngineMessage>) -> Result<Self> {
    let Certs { key, certs } = get_certs()?;
    let home_path = dirs::home_dir().unwrap().join("adb");
    let cache_dir = home_path.join("cache");

    create_dir_all(&cache_dir)?;

    let db_path = home_path.join("dns-adblock.sqlite");
    let db = DB::init(db_path).await?;

    let config_path = home_path.join("config.toml");
    let mut config = Config::from_file(&config_path)?;
    config.compile_regexes()?;

    let mut r_config = ResolverConfig::default();

    r_config.add_name_server(NameServerConfig::https(
      IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
      Arc::from("cloudflare-dns.com"),
      None,
    ));

    r_config.add_name_server(NameServerConfig::https(
      IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1)),
      Arc::from("cloudflare-dns.com"),
      None,
    ));

    r_config.add_name_server(NameServerConfig::https(
      IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
      Arc::from("dns.google"),
      None,
    ));

    let mut resolver_builder =
      TokioResolver::builder_with_config(r_config, TokioRuntimeProvider::default());

    let opts = resolver_builder.options_mut();
    opts.negative_min_ttl = Some(Duration::from_secs(60));
    opts.positive_min_ttl = Some(Duration::from_secs(60));
    opts.num_concurrent_reqs = 3;

    let server_config = Arc::new(
      ServerConfig::builder().with_no_client_auth().with_single_cert(certs, key)?,
    );

    let paths = download_mmdbs_files();

    let ctx = Self(Arc::new(ContextImpl {
      tx,
      config: RwLock::new(config),
      db,
      resolver: resolver_builder.build()?,
      ws_tx: broadcast::channel(100).0,
      cache_dir,
      server_config,
      config_path,
      mmdbs: RwLock::new(None),
      paths,
      dns_cache: DnsCache::new(),
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

  pub fn resolver(&self) -> &TokioResolver {
    &self.0.resolver
  }

  pub fn blocklists(&self) -> Vec<String> {
    self.0.config.read().blocklists.clone()
  }

  pub fn block_rules(&self) -> Option<Vec<String>> {
    self.0.config.read().block_rules.clone()
  }

  pub fn socket() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 53))
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

  pub fn cache(&self) -> &DnsCache {
    &self.0.dns_cache
  }
}
