use crate::database::DB;
use crate::domain::{query_domain, registered_domain};
use crate::engine::BlockOrigin;
use chrono::Utc;
use hickory_proto::op::Message;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use tokio::sync::mpsc::Receiver;
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
  pub device: Option<String>,
}

impl QueryEvent {
  pub fn new(
    domain: String,
    client_ip: String,
    blocked: bool,
    block_origin: BlockOrigin,
    response_time: i64,
    device: Option<String>,
  ) -> Self {
    Self {
      registered_domain: registered_domain(&domain),
      domain,
      client_ip,
      blocked,
      block_origin,
      timestamp: Utc::now().timestamp(),
      response_time,
      device,
    }
  }
}

impl DB {
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
      "INSERT INTO query_log (domain, client_ip, blocked, block_origin, timestamp, response_time, device_id) VALUES (?, ?, ?, ?, ?, ?, ?)",
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
      .bind(&event.device)
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

  pub fn record_query(
    &self,
    response: &Message,
    src: SocketAddr,
    blocked: bool,
    block_origin: BlockOrigin,
    response_time: i64,
    device: Option<String>,
  ) {
    if let Some(domain) = query_domain(response) {
      let event = QueryEvent::new(
        domain,
        src.ip().to_string(),
        blocked,
        block_origin,
        response_time,
        device,
      );

      let _ = self
        .record_tx
        .as_ref()
        .expect("Should always exist when running from a daemon")
        .try_send(event);
    }
  }
}
