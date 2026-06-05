pub mod devices;
pub mod query_logs;
pub mod schema;
pub mod stats;

use crate::database::query_logs::QueryEvent;
use crate::domain::{query_domain, registered_domain};
use crate::engine::BlockOrigin;
use anyhow::{anyhow, Result};
use chrono::Duration as ChronoDuration;
use chrono::{Timelike, Utc};
use dashmap::DashSet;
use hickory_proto::op::Message;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct DB {
  pub pool: SqlitePool,
  pub total_queries: Arc<AtomicUsize>,
  pub record_tx: Option<Sender<QueryEvent>>,
  pub known_devices: DashSet<String>,
}

impl DB {
  pub async fn init<P: AsRef<Path>>(db_path: P) -> Result<Self> {
    let pool = Self::init_db(db_path.as_ref()).await?;
    let (tx, rx) = channel(10000);

    let db = Self { pool, total_queries: Default::default(), record_tx: Some(tx), known_devices: DashSet::new() };

    db.init_schema().await?;
    db.spawn_inserter(rx);
    db.populate_devices().await?;

    Ok(db)
  }

  pub async fn populate_devices(&self) -> Result<()> {
    let devices = self.get_devices().await.map_err(|err| anyhow!("{err}"))?;
    let ids = devices.iter().map(|d| d.id.clone()).collect::<Vec<_>>();

    for id in ids {
      self.known_devices.insert(id);
    }

    Ok(())
  }

  pub async fn init_simple<P: AsRef<Path>>(db_path: P) -> Result<Self> {
    let pool = Self::init_db(db_path.as_ref()).await?;

    let db = Self { pool, total_queries: Default::default(), record_tx: None, known_devices: DashSet::new() };
    db.init_schema().await?;

    Ok(db)
  }

  pub async fn spawn_cleanup_task(
    pool: SqlitePool,
    retention: ChronoDuration,
  ) -> Result<()> {
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
