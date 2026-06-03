use crate::ChronoDuration;
use crate::domain::{query_domain, registered_domain};
use crate::engine::BlockOrigin;
use chrono::{Timelike, Utc};
use hickory_proto::op::Message;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tracing::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEvent {
  pub domain: String,
  pub registered_domain: String,
  pub client_ip: String,
  pub blocked: bool,
  pub block_origin: BlockOrigin,
  pub timestamp: i64,
  pub response_time: i64,
}

impl QueryEvent {
  pub fn new(
    domain: String,
    client_ip: String,
    blocked: bool,
    block_origin: BlockOrigin,
    response_time: i64,
  ) -> Self {
    Self {
      registered_domain: registered_domain(&domain),
      domain,
      client_ip,
      blocked,
      block_origin,
      timestamp: Utc::now().timestamp(),
      response_time,
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
  pub last_seen: i64,
  pub avg_response_time: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stats {
  pub total_queries: usize,
  pub total_blocked: i64,
  pub total_allowed: i64,
  pub block_rate: f64,
  pub avg_response_time: f64,
}

#[derive(Debug, Clone)]
pub struct DB {
  pub pool: SqlitePool,
  pub total_queries: Arc<AtomicUsize>,
  pub record_tx: Sender<QueryEvent>,
}

impl DB {
  pub async fn init<P: AsRef<Path>>(db_path: P) -> anyhow::Result<Self> {
    let pool = Self::init_db(db_path.as_ref()).await?;
    let (tx, rx) = channel(10000);

    let db = Self { pool, total_queries: Default::default(), record_tx: tx };
    db.init_schema().await?;
    db.spawn_inserter(rx);

    Ok(db)
  }

  pub fn spawn_inserter(&self, mut rx: Receiver<QueryEvent>) {
    let db = self.clone();
    tokio::spawn(async move {
      while let Some(event) = rx.recv().await {
        debug!(
          "dns request: {}ms blocked={} src={}",
          event.response_time, event.blocked, event.domain
        );
        if let Err(err) = db.insert_query(&event).await {
          warn!(error = ?err, "failed to insert query_log");
        }
      }
    });
  }

  pub async fn insert_query(&self, event: &QueryEvent) -> anyhow::Result<()> {
    let mut tx = self.pool.begin().await?;

    sqlx::query(
      "INSERT INTO query_log (domain, client_ip, blocked, block_origin, timestamp, response_time) VALUES (?, ?, ?, ?, ?, ?)",
    )
      .bind(&event.domain)
      .bind(&event.client_ip)
      .bind(event.blocked)
      .bind(match event.block_origin {
        BlockOrigin::Plain => "plain",
        BlockOrigin::DoH => "doh",
        BlockOrigin::DoT => "dot",
      })
      .bind(event.timestamp)
      .bind(event.response_time)
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

    self.total_queries.fetch_add(1, Ordering::Relaxed);

    Ok(())
  }

  pub async fn latest_blocked(&self, limit: i64) -> anyhow::Result<Vec<BlockedEntry>> {
    let rows = sqlx::query_as::<_, BlockedEntry>(
      "SELECT domain, client_ip, timestamp
             FROM query_log
             WHERE blocked = 1
             ORDER BY timestamp DESC
             LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&self.pool)
    .await?;

    Ok(rows)
  }

  pub async fn top_blocked(&self, limit: Option<i64>) -> anyhow::Result<Vec<TopDomain>> {
    let base = "
        SELECT
            ds.domain,
            ds.hits_blocked,
            ds.hits_total,
            ds.last_seen,
            COALESCE(AVG(ql.response_time), 0.0) AS avg_response_time
        FROM domain_stats ds
        LEFT JOIN query_log ql ON ql.domain = ds.domain
        WHERE ds.hits_blocked > 0
        GROUP BY ds.domain
        ORDER BY ds.hits_blocked DESC";

    let rows = match limit {
      Some(limit) => {
        sqlx::query_as::<_, TopDomain>(&format!("{base} LIMIT ?"))
          .bind(limit)
          .fetch_all(&self.pool)
          .await?
      }
      None => sqlx::query_as::<_, TopDomain>(base).fetch_all(&self.pool).await?,
    };

    Ok(rows)
  }

  pub async fn stats(
    &self,
    since: Option<ChronoDuration>,
    until: Option<ChronoDuration>,
  ) -> anyhow::Result<Stats> {
    let since_ts = since.map(|d| (Utc::now() - d).timestamp());
    let until_ts = until.map(|d| (Utc::now() - d).timestamp());

    let total_blocked: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM query_log WHERE blocked = 1
         AND (? IS NULL OR timestamp >= ?)
         AND (? IS NULL OR timestamp <= ?)",
    )
    .bind(since_ts)
    .bind(since_ts)
    .bind(until_ts)
    .bind(until_ts)
    .fetch_one(&self.pool)
    .await?;

    let total_allowed: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM query_log WHERE blocked = 0
         AND (? IS NULL OR timestamp >= ?)
         AND (? IS NULL OR timestamp <= ?)",
    )
    .bind(since_ts)
    .bind(since_ts)
    .bind(until_ts)
    .bind(until_ts)
    .fetch_one(&self.pool)
    .await?;

    let avg_response_time: Option<f64> = sqlx::query_scalar(
      "SELECT AVG(response_time) FROM query_log
         WHERE (? IS NULL OR timestamp >= ?)
         AND (? IS NULL OR timestamp <= ?)",
    )
    .bind(since_ts)
    .bind(since_ts)
    .bind(until_ts)
    .bind(until_ts)
    .fetch_one(&self.pool)
    .await?;

    let total = total_blocked + total_allowed;

    let block_rate =
      if total > 0 { total_blocked as f64 / total as f64 * 100.0 } else { 0.0 };

    Ok(Stats {
      total_queries: total as usize,
      total_blocked,
      total_allowed,
      block_rate,
      avg_response_time: avg_response_time.unwrap_or(0.0),
    })
  }

  pub async fn spawn_cleanup_task(
    pool: SqlitePool,
    retention: ChronoDuration,
  ) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(24 * 60 * 60));

    loop {
      interval.tick().await;

      let cutoff = Utc::now() - retention;
      if let Err(err) = sqlx::query("DELETE FROM query_log WHERE timestamp < ?")
        .bind(cutoff.timestamp())
        .execute(&pool)
        .await
      {
        warn!(error = ?err, "failed to cleanup query_log");
      }
    }

    Ok(())
  }

  // TODO: possibly use channels for this
  pub fn record_query(
    &self,
    response: &Message,
    src: SocketAddr,
    blocked: bool,
    block_origin: BlockOrigin,
    response_time: i64,
  ) {
    if let Some(domain) = query_domain(response) {
      let event = QueryEvent::new(
        domain,
        src.ip().to_string(),
        blocked,
        block_origin,
        response_time,
      );

      let _ = self.record_tx.try_send(event);
    }
  }

  async fn init_db(path: &Path) -> anyhow::Result<SqlitePool> {
    let options = SqliteConnectOptions::new().filename(path).create_if_missing(true);

    Ok(SqlitePoolOptions::new().max_connections(5).connect_with(options).await?)
  }

  async fn init_schema(&self) -> anyhow::Result<()> {
    sqlx::query(
      "CREATE TABLE IF NOT EXISTS query_log (
         id            INTEGER PRIMARY KEY AUTOINCREMENT,
         domain        TEXT    NOT NULL,
         client_ip     TEXT    NOT NULL,
         blocked       INTEGER NOT NULL,
         block_origin  TEXT,
         timestamp     INTEGER NOT NULL,
         response_time INTEGER NOT NULL
       )",
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      "CREATE INDEX IF NOT EXISTS idx_query_log_blocked_timestamp
             ON query_log(blocked, timestamp)",
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      "CREATE INDEX IF NOT EXISTS idx_query_log_domain
             ON query_log(domain)",
    )
    .execute(&self.pool)
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
    .execute(&self.pool)
    .await?;

    sqlx::query(
      "CREATE INDEX IF NOT EXISTS idx_domain_stats_registered
                ON domain_stats(registered_domain);",
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      "CREATE INDEX IF NOT EXISTS idx_domain_stats_last_seen
             ON domain_stats(last_seen)",
    )
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  pub async fn stats_by_hour_today(&self) -> anyhow::Result<Vec<HourStat>> {
    let now = Utc::now();
    let start_of_day =
      now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();

    let rows = sqlx::query_as::<_, HourStat>(
      "WITH RECURSIVE hours(h) AS (
       SELECT 0
       UNION ALL
       SELECT h + 1 FROM hours WHERE h < ?
     )
     SELECT
       printf('%02d:00', hours.h)        AS hour,
       COALESCE(COUNT(q.id),   0)        AS total,
       COALESCE(SUM(q.blocked), 0)       AS blocked
     FROM hours
     LEFT JOIN query_log q
       ON  strftime('%H', q.timestamp, 'unixepoch') = printf('%02d', hours.h)
       AND q.timestamp >= ?
     GROUP BY hours.h
     ORDER BY hours.h ASC",
    )
    .bind(23i64)
    .bind(start_of_day)
    .fetch_all(&self.pool)
    .await?;

    Ok(rows)
  }
}

#[derive(sqlx::FromRow)]
pub struct HourStat {
  pub hour: String,
  pub total: i64,
  pub blocked: i64,
}
