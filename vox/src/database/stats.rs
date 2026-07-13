use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::AssertSqlSafe;

use crate::database::DB;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct HourStat {
  pub hour: String,
  pub total: i64,
  pub blocked: i64,
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

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct CountryStat {
  pub country_code: String,
  pub total: i64,
  pub blocked: i64,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct PopularStat {
  pub label: String,
  pub total: i64,
  pub blocked: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsChange {
  pub total_queries: f64,
  pub total_blocked: f64,
  pub total_allowed: f64,
  pub block_rate: f64,
  pub avg_response_time: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
  pub total_queries: usize,
  pub total_blocked: i64,
  pub total_allowed: i64,
  pub block_rate: f64,
  pub avg_response_time: f64,
  pub top_countries: Vec<CountryStat>,
  pub top_companies: Vec<PopularStat>,
  pub weekly_change: Option<StatsChange>,
}

impl DB {
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
        sqlx::query_as::<_, TopDomain>(AssertSqlSafe(format!("{base} LIMIT ?")))
          .bind(limit)
          .fetch_all(&self.pool)
          .await?
      }
      None => sqlx::query_as::<_, TopDomain>(base).fetch_all(&self.pool).await?,
    };

    Ok(rows)
  }

  async fn stats_scalars(
    &self,
    since_ts: Option<i64>,
    until_ts: Option<i64>,
  ) -> anyhow::Result<(i64, i64, f64)> {
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

    Ok((total_blocked, total_allowed, avg_response_time.unwrap_or(0.0)))
  }

  pub async fn stats(
    &self,
    since: Option<Duration>,
    until: Option<Duration>,
  ) -> anyhow::Result<Stats> {
    let now = Utc::now();
    let since_ts = since.map(|d| (now - d).timestamp());
    let until_ts = until.map(|d| (now - d).timestamp());

    let (total_blocked, total_allowed, avg_response_time) =
      self.stats_scalars(since_ts, until_ts).await?;

    let top_countries = sqlx::query_as::<_, CountryStat>(
      "SELECT country_code,
                    COUNT(*) AS total,
                    COALESCE(SUM(blocked), 0) AS blocked
               FROM query_log
               WHERE country_code IS NOT NULL
                 AND country_code != ''
                 AND (? IS NULL OR timestamp >= ?)
                 AND (? IS NULL OR timestamp <= ?)
               GROUP BY country_code
               ORDER BY total DESC, country_code ASC
               LIMIT 5",
    )
    .bind(since_ts)
    .bind(since_ts)
    .bind(until_ts)
    .bind(until_ts)
    .fetch_all(&self.pool)
    .await?;

    let top_companies = sqlx::query_as::<_, PopularStat>(
      "SELECT company_name AS label,
                    COUNT(*) AS total,
                    COALESCE(SUM(blocked), 0) AS blocked
               FROM query_log
               WHERE company_name IS NOT NULL
                 AND company_name != ''
                 AND (? IS NULL OR timestamp >= ?)
                 AND (? IS NULL OR timestamp <= ?)
               GROUP BY company_name
               ORDER BY total DESC, label ASC
               LIMIT 5",
    )
    .bind(since_ts)
    .bind(since_ts)
    .bind(until_ts)
    .bind(until_ts)
    .fetch_all(&self.pool)
    .await?;

    let total = total_blocked + total_allowed;
    let block_rate =
      if total > 0 { total_blocked as f64 / total as f64 * 100.0 } else { 0.0 };

    let now_ts = now.timestamp();

    let current_week_start = now_ts - Duration::weeks(1).num_seconds();
    let previous_week_start = now_ts - Duration::weeks(2).num_seconds();

    let (week_blocked, week_allowed, week_avg_rt) =
      self.stats_scalars(Some(current_week_start), Some(now_ts)).await?;

    let (prev_blocked, prev_allowed, prev_avg_rt) =
      self.stats_scalars(Some(previous_week_start), Some(current_week_start)).await?;

    let week_total = week_blocked + week_allowed;
    let prev_total = prev_blocked + prev_allowed;

    let week_block_rate =
      if week_total > 0 { week_blocked as f64 / week_total as f64 * 100.0 } else { 0.0 };

    let prev_block_rate =
      if prev_total > 0 { prev_blocked as f64 / prev_total as f64 * 100.0 } else { 0.0 };

    let change = if prev_total > 0 {
      Some(StatsChange {
        total_queries: (week_total - prev_total) as f64 / prev_total as f64 * 100.0,

        total_blocked: if prev_blocked > 0 {
          (week_blocked - prev_blocked) as f64 / prev_blocked as f64 * 100.0
        } else if week_blocked > 0 {
          100.0
        } else {
          0.0
        },

        total_allowed: if prev_allowed > 0 {
          (week_allowed - prev_allowed) as f64 / prev_allowed as f64 * 100.0
        } else if week_allowed > 0 {
          100.0
        } else {
          0.0
        },

        block_rate: week_block_rate - prev_block_rate,

        avg_response_time: if prev_avg_rt > 0.0 {
          (week_avg_rt - prev_avg_rt) / prev_avg_rt * 100.0
        } else {
          0.0
        },
      })
    } else {
      None
    };

    Ok(Stats {
      total_queries: total as usize,
      total_blocked,
      total_allowed,
      block_rate,
      avg_response_time,
      top_countries,
      top_companies,
      weekly_change: change,
    })
  }
}
