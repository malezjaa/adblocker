use crate::blocker::BlockLookup;
use crate::config::Config;
use crate::domain::{query_domain, registered_domain};
use anyhow::Result;
use chrono::{Duration as ChronoDuration, Utc};
use hickory_proto::op::Message;
use hickory_resolver::config::{ResolverConfig, CLOUDFLARE};
use hickory_resolver::{net::runtime::TokioRuntimeProvider, TokioResolver};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct State(Arc<StateImpl>);

#[derive(Debug)]
pub struct StateImpl {
  tx: Sender<BlockLookup>,
  config: RwLock<Config>,
  db: SqlitePool,
  total_queries: AtomicUsize,
  resolver: TokioResolver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEvent {
  pub domain: String,
  pub registered_domain: String,
  pub client_ip: String,
  pub blocked: bool,
  pub timestamp: i64,
}

impl QueryEvent {
  pub fn new(domain: String, client_ip: String, blocked: bool) -> Self {
    Self {
      registered_domain: registered_domain(&domain),
      domain,
      client_ip,
      blocked,
      timestamp: Utc::now().timestamp(),
    }
  }
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct BlockedEntry {
  pub domain: String,
  pub client_ip: String,
  pub timestamp: i64,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct TopDomain {
  pub domain: String,
  pub hits_blocked: i64,
  pub hits_total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaySummary {
  pub total_queries: usize,
  pub total_blocked: i64,
  pub total_allowed: i64,
  pub block_rate: f64,
}

impl State {
  pub fn new(config: Config, db: SqlitePool, tx: Sender<BlockLookup>) -> Result<Self> {
    let resolver =
      TokioResolver::builder_with_config(ResolverConfig::udp_and_tcp(&CLOUDFLARE), TokioRuntimeProvider::default()).build()?;

    Ok(Self(Arc::new(StateImpl {
      tx,
      config: RwLock::new(config),
      db,
      total_queries: AtomicUsize::default(),
      resolver,
    })))
  }

  pub async fn from_paths<P: AsRef<Path>, Q: AsRef<Path>>(
    config_path: P,
    db_path: Q,
    tx: Sender<BlockLookup>,
  ) -> Result<Self> {
    let config = Config::from_file(config_path)?;
    let db = Self::init_db(db_path.as_ref()).await?;
    let state = Self::new(config, db, tx)?;
    state.init_schema().await?;
    Ok(state)
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

  pub async fn record_query(&self, event: &QueryEvent) -> Result<()> {
    let mut tx = self.0.db.begin().await?;

    sqlx::query(
      "INSERT INTO query_log (domain, client_ip, blocked, timestamp) VALUES (?, ?, ?, ?)",
    )
      .bind(&event.domain)
      .bind(&event.client_ip)
      .bind(event.blocked)
      .bind(event.timestamp)
      .execute(&mut *tx)
      .await?;

    let hits_blocked = i64::from(event.blocked);

    sqlx::query(
      "INSERT INTO domain_stats (domain, registered_domain, hits_total, hits_blocked, last_seen)
             VALUES (?, ?, 1, ?, ?)
             ON CONFLICT(domain) DO UPDATE SET
               hits_total        = hits_total + 1,
               hits_blocked      = hits_blocked + excluded.hits_blocked,
               last_seen         = excluded.last_seen",
    )
      .bind(&event.domain)
      .bind(&event.registered_domain)
      .bind(hits_blocked)
      .bind(event.timestamp)
      .execute(&mut *tx)
      .await?;

    tx.commit().await?;

    self.0.total_queries.fetch_add(1, Ordering::Relaxed);

    Ok(())
  }

  /// The most recently blocked domains, newest first.
  pub async fn latest_blocked(&self, limit: i64) -> Result<Vec<BlockedEntry>> {
    let rows = sqlx::query_as::<_, BlockedEntry>(
      "SELECT domain, client_ip, timestamp
             FROM query_log
             WHERE blocked = 1
             ORDER BY timestamp DESC
             LIMIT ?",
    )
      .bind(limit)
      .fetch_all(&self.0.db)
      .await?;

    Ok(rows)
  }

  /// Domains with the most blocked hits, highest first.
  pub async fn top_blocked(&self, limit: i64) -> Result<Vec<TopDomain>> {
    let rows = sqlx::query_as::<_, TopDomain>(
      "SELECT domain, registered_domain, hits_blocked, hits_total
             FROM domain_stats
             WHERE hits_blocked > 0
             ORDER BY hits_blocked DESC
             LIMIT ?",
    )
      .bind(limit)
      .fetch_all(&self.0.db)
      .await?;

    Ok(rows)
  }

  pub async fn summary_today(&self) -> Result<DaySummary> {
    let since = (Utc::now() - ChronoDuration::hours(24)).timestamp();

    let total_blocked: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM query_log WHERE blocked = 1 AND timestamp >= ?",
    )
      .bind(since)
      .fetch_one(&self.0.db)
      .await?;

    let total_allowed: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM query_log WHERE blocked = 0 AND timestamp >= ?",
    )
      .bind(since)
      .fetch_one(&self.0.db)
      .await?;

    let total = total_blocked + total_allowed;
    let block_rate =
      if total > 0 { total_blocked as f64 / total as f64 * 100.0 } else { 0.0 };

    Ok(DaySummary {
      total_queries: self.0.total_queries.load(Ordering::Relaxed),
      total_blocked,
      total_allowed,
      block_rate,
    })
  }

  pub fn spawn_cleanup_task(&self, retention: ChronoDuration) -> JoinHandle<()> {
    let db = self.0.db.clone();

    tokio::spawn(async move {
      let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(24 * 60 * 60));

      loop {
        interval.tick().await;

        let cutoff = Utc::now() - retention;
        if let Err(err) = sqlx::query("DELETE FROM query_log WHERE timestamp < ?")
          .bind(cutoff.timestamp())
          .execute(&db)
          .await
        {
          warn!(error = ?err, "failed to cleanup query_log");
        }
      }
    })
  }

  pub fn spawn_query_record(&self, response: &Message, src: SocketAddr, blocked: bool) {
    if let Some(domain) = query_domain(response) {
      let event = QueryEvent::new(domain, src.ip().to_string(), blocked);

      let state = self.clone();
      tokio::spawn(async move {
        if let Err(err) = state.record_query(&event).await {
          warn!(error = ?err, "failed to insert query_log");
        }
      });
    }
  }

  async fn init_db(path: &Path) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::new().filename(path).create_if_missing(true);

    Ok(SqlitePoolOptions::new().max_connections(5).connect_with(options).await?)
  }

  async fn init_schema(&self) -> Result<()> {
    sqlx::query(
      "CREATE TABLE IF NOT EXISTS query_log (
               id        INTEGER PRIMARY KEY AUTOINCREMENT,
               domain    TEXT    NOT NULL,
               client_ip TEXT    NOT NULL,
               blocked   INTEGER NOT NULL,
               timestamp INTEGER NOT NULL
             )",
    )
      .execute(&self.0.db)
      .await?;

    sqlx::query(
      "CREATE INDEX IF NOT EXISTS idx_query_log_blocked_timestamp
             ON query_log(blocked, timestamp)",
    )
      .execute(&self.0.db)
      .await?;

    sqlx::query(
      "CREATE INDEX IF NOT EXISTS idx_query_log_domain
             ON query_log(domain)",
    )
      .execute(&self.0.db)
      .await?;

    sqlx::query(
      "CREATE TABLE IF NOT EXISTS domain_stats (
              domain             TEXT    PRIMARY KEY,
              registered_domain  TEXT    NOT NULL,
              hits_total         INTEGER NOT NULL,
              hits_blocked       INTEGER NOT NULL,
              last_seen          INTEGER NOT NULL
            );",
    )
      .execute(&self.0.db)
      .await?;

    sqlx::query(
      "CREATE INDEX IF NOT EXISTS idx_domain_stats_registered
                ON domain_stats(registered_domain);",
    )
      .execute(&self.0.db)
      .await?;

    sqlx::query(
      "CREATE INDEX IF NOT EXISTS idx_domain_stats_last_seen
             ON domain_stats(last_seen)",
    )
      .execute(&self.0.db)
      .await?;

    Ok(())
  }
}
