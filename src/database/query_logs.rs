use crate::context::Context;
use crate::dashboard::QueryLog;
use crate::database::DB;
use crate::database::devices::{Device, DeviceType};
use crate::domain::{query_domain, registered_domain};
use crate::engine::message::BlockOrigin;
use anyhow::Result;
use chrono::Utc;
use clap::ValueEnum;
use hickory_proto::op::Message;
use hickory_proto::rr::RData;
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
  pub resolved_ip: Option<String>,
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
    resolved_ip: Option<String>,
    blocked: bool,
    block_origin: BlockOrigin,
    response_time: i64,
    device: Option<String>,
  ) -> Self {
    Self {
      registered_domain: registered_domain(&domain),
      domain,
      client_ip,
      resolved_ip,
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

  pub async fn insert_query(&self, event: &QueryEvent) -> Result<()> {
    let mut tx = self.pool.begin().await?;

    let ctx = self.context();

    let mut resolved_ip = event.resolved_ip.clone();
    if resolved_ip.is_none() && event.blocked {
      resolved_ip = resolve_domain_ip(ctx.as_ref(), &event.domain).await;
    }

    let (country_code, company_name) =
      lookup_geo_company(ctx.as_ref(), resolved_ip.as_deref());

    let device = if let Some(device_id) = &event.device {
      if let Some(id) =
        self.known_devices.iter().find(|d| d.to_lowercase() == device_id.to_lowercase())
      {
        Some(id.clone())
      } else {
        warn!("Received query for unknown device: {}", device_id);
        None
      }
    } else {
      None
    };

    sqlx::query(
      "INSERT INTO query_log (domain, client_ip, blocked, block_origin, timestamp, response_time, device_id, country_code, company_name) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
      .bind(&event.domain)
      .bind(&event.client_ip)
      .bind(event.blocked)
      .bind(match event.block_origin {
        BlockOrigin::Plain => "plain",
        BlockOrigin::DoH => "doh",
      })
      .bind(event.timestamp)
      .bind(event.response_time)
      .bind(&device)
      .bind(country_code)
      .bind(company_name)
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
    if let Some(ref device_id) = device {
      sqlx::query("UPDATE device SET last_seen = ? WHERE id = ?")
        .bind(event.timestamp)
        .bind(device_id)
        .execute(&mut *tx)
        .await?;
    }

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
      let resolved_ip = if blocked { None } else { first_answer_ip(response) };

      let event = QueryEvent::new(
        domain,
        src.ip().to_string(),
        resolved_ip,
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

  pub async fn query_logs(
    &self,
    page: u32,
    per_page: u32,
    domain_filter: Option<&str>,
  ) -> Result<(Vec<QueryLog>, i64)> {
    let offset = ((page - 1) * per_page) as i64;

    let total: i64 = match domain_filter {
      Some(domain) => {
        let pattern = format!("%{}%", domain);
        sqlx::query_scalar("SELECT COUNT(*) FROM query_log WHERE domain LIKE ?")
          .bind(pattern)
          .fetch_one(&self.pool)
          .await?
      }
      None => {
        sqlx::query_scalar("SELECT COUNT(*) FROM query_log").fetch_one(&self.pool).await?
      }
    };

    #[derive(sqlx::FromRow)]
    struct QueryLogRow {
      pub id: i64,
      pub domain: String,
      pub client_ip: String,
      pub blocked: bool,
      pub block_origin: Option<String>,
      pub timestamp: i64,
      pub response_time: i64,
      pub country_code: Option<String>,
      pub company_name: Option<String>,

      pub device_id: Option<String>,
      pub device_name: Option<String>,
      pub device_type: Option<String>,
      pub device_last_seen: Option<i64>,
    }

    let rows = match domain_filter {
      Some(domain) => {
        let pattern = format!("%{}%", domain);
        sqlx::query_as::<_, QueryLogRow>(
          r#"
                SELECT
                    q.id,
                    q.domain,
                    q.client_ip,
                    q.blocked,
                    q.block_origin,
                    q.timestamp,
                    q.response_time,
                    q.country_code,
                    q.company_name,

                    d.id AS device_id,
                    d.name AS device_name,
                    d.type AS device_type,
                    d.last_seen AS device_last_seen
                FROM query_log q
                LEFT JOIN device d ON q.device_id = d.id
                WHERE q.domain LIKE ?
                ORDER BY q.timestamp DESC
                LIMIT ?
                OFFSET ?
                "#,
        )
        .bind(pattern)
        .bind(per_page as i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?
      }
      None => {
        sqlx::query_as::<_, QueryLogRow>(
          r#"
                SELECT
                    q.id,
                    q.domain,
                    q.client_ip,
                    q.blocked,
                    q.block_origin,
                    q.timestamp,
                    q.response_time,
                    q.country_code,
                    q.company_name,

                    d.id AS device_id,
                    d.name AS device_name,
                    d.type AS device_type,
                    d.last_seen AS device_last_seen
                FROM query_log q
                LEFT JOIN device d ON q.device_id = d.id
                ORDER BY q.timestamp DESC
                LIMIT ?
                OFFSET ?
                "#,
        )
        .bind(per_page as i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?
      }
    };

    let logs = rows
      .into_iter()
      .map(|row| QueryLog {
        id: row.id,
        domain: row.domain,
        client_ip: row.client_ip,
        blocked: row.blocked,
        block_origin: row.block_origin,
        timestamp: row.timestamp,
        response_time: row.response_time,
        country_code: row.country_code,
        company_name: row.company_name,

        device: row.device_id.map(|id| Device {
          id,
          name: row.device_name.unwrap(),
          device_type: DeviceType::from_str(&row.device_type.unwrap(), true).unwrap(),
          last_seen: row.device_last_seen.unwrap(),
        }),
      })
      .collect();

    Ok((logs, total))
  }
}

fn first_answer_ip(response: &Message) -> Option<String> {
  response.answers.iter().find_map(|record| match &record.data {
    RData::A(ip) => Some(ip.0.to_string()),
    RData::AAAA(ip) => Some(ip.0.to_string()),
    _ => None,
  })
}

fn lookup_geo_company(
  ctx: Option<&Context>,
  ip: Option<&str>,
) -> (Option<String>, Option<String>) {
  let (Some(ctx), Some(ip)) = (ctx, ip) else {
    return (None, None);
  };

  match ctx.lookup_mmdb(ip.to_string()) {
    Ok(Some(geo)) => (geo.country, geo.asn_org),
    _ => (None, None),
  }
}

async fn resolve_domain_ip(ctx: Option<&Context>, domain: &str) -> Option<String> {
  let ctx = ctx?;
  let lookup = ctx.resolver().lookup_ip(domain.trim_end_matches('.')).await.ok()?;
  lookup.iter().next().map(|ip| ip.to_string())
}
