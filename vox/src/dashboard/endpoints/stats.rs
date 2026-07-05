use crate::context::Context;
use crate::dashboard::AppError;
use crate::dashboard::auth::AuthGuard;
use crate::database::devices::Device;
use crate::database::stats::{Stats, TopDomain};
use axum::Json;
use axum::extract::Query;
use axum::extract::State as AxumState;
use chrono::Duration;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Limit {
  limit: Option<i64>,
}

pub async fn top_handler(
  _guard: AuthGuard,
  AxumState(ctx): AxumState<Context>,
  Query(limit): Query<Limit>,
) -> anyhow::Result<Json<Vec<TopDomain>>, AppError> {
  let top = ctx.db().top_blocked(limit.limit).await?;
  Ok(Json(top))
}

#[derive(Deserialize)]
pub struct StatsQuery {
  since: Option<Duration>,
  until: Option<Duration>,
}

pub async fn stats(
  _guard: AuthGuard,
  AxumState(ctx): AxumState<Context>,
  Query(query): Query<StatsQuery>,
) -> anyhow::Result<Json<Stats>, AppError> {
  let stats = ctx.db().stats(query.since, query.until).await?;
  Ok(Json(stats))
}

#[derive(Serialize, Deserialize)]
pub struct ChartData {
  pub hour: String,
  pub total: i64,
  pub blocked: i64,
}

pub async fn chart_data(
  _guard: AuthGuard,
  AxumState(ctx): AxumState<Context>,
) -> anyhow::Result<Json<Vec<ChartData>>, AppError> {
  let rows = ctx.db().stats_by_hour_today().await?;
  let data = rows
    .into_iter()
    .map(|r| ChartData { hour: r.hour, total: r.total, blocked: r.blocked })
    .collect();
  Ok(Json(data))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct QueryLog {
  pub id: i64,
  pub domain: String,
  pub record_type: String,
  pub client_ip: String,
  pub blocked: bool,
  pub block_origin: Option<u8>,
  pub response_code: String,
  pub timestamp: i64,
  pub response_time: i64,
  pub country_code: Option<String>,
  pub company_name: Option<String>,

  pub device: Option<Device>,
}

#[derive(Serialize)]
pub struct PaginatedQueryLogs {
  total: i64,
  page: u32,
  per_page: u32,
  items: Vec<QueryLog>,
}

#[derive(Deserialize)]
pub struct QueryLogsQuery {
  page: Option<u32>,
  per_page: Option<u32>,
  domain: Option<String>,
}

pub async fn query_logs_handler(
  _guard: AuthGuard,
  AxumState(ctx): AxumState<Context>,
  Query(query): Query<QueryLogsQuery>,
) -> anyhow::Result<Json<PaginatedQueryLogs>, AppError> {
  let page = query.page.unwrap_or(1).max(1);
  let per_page = query.per_page.unwrap_or(50).clamp(1, 500);

  let (items, total) =
    ctx.db().query_logs(page, per_page, query.domain.as_deref()).await?;

  Ok(Json(PaginatedQueryLogs { total, page, per_page, items }))
}
