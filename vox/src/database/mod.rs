pub mod admin;
pub mod devices;
pub mod query_logs;
pub mod schema;
pub mod sessions;
pub mod stats;

use crate::context::{Context, ContextImpl};
use crate::database::query_logs::QueryEvent;
use anyhow::{Result, anyhow};
use chrono::Duration as ChronoDuration;
use chrono::Utc;
use dashmap::DashSet;
use parking_lot::RwLock;
use sqlx::SqlitePool;
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Weak};
use std::time::Duration;
use tokio::sync::mpsc::{Sender, channel};
use tracing::warn;

#[derive(Debug, Clone)]
pub struct DB {
  pub pool: SqlitePool,
  pub total_queries: Arc<AtomicUsize>,
  pub record_tx: Option<Sender<QueryEvent>>,
  pub known_devices: DashSet<String>,
  pub ctx_ref: Arc<RwLock<Option<Weak<ContextImpl>>>>,
}

impl DB {
  pub async fn init<P: AsRef<Path>>(db_path: P) -> Result<Self> {
    let pool = Self::init_db(db_path.as_ref()).await?;
    let (tx, rx) = channel(10000);

    let db = Self {
      pool,
      total_queries: Default::default(),
      record_tx: Some(tx),
      known_devices: DashSet::new(),
      ctx_ref: Default::default(),
    };

    db.init_schema().await?;
    db.populate_devices().await?;
    db.spawn_inserter(rx);

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

    let db = Self {
      pool,
      total_queries: Default::default(),
      record_tx: None,
      known_devices: DashSet::new(),
      ctx_ref: Default::default(),
    };
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
  }

  pub async fn reset_stats(&self) -> Result<()> {
    let mut tx = self.pool.begin().await?;

    sqlx::query("DELETE FROM query_log").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM domain_stats").execute(&mut *tx).await?;

    tx.commit().await?;

    Ok(())
  }

  pub fn attach_context(&self, ctx: &Context) {
    *self.ctx_ref.write() = Some(ctx.downgrade());
  }

  pub fn context(&self) -> Option<Context> {
    Context::from_weak(self.ctx_ref.read().as_ref()?)
  }
}
