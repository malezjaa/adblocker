pub mod devices;
pub mod query_logs;
pub mod schema;
pub mod stats;

use crate::ChronoDuration;
use crate::database::query_logs::QueryEvent;
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
}
