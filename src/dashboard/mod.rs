pub mod app_error;
pub mod frontend;
pub mod ws;

use crate::application::app::App;
use crate::context::Context;
pub use crate::dashboard::app_error::AppError;
use crate::dashboard::frontend::serve_file;
use crate::dashboard::ws::ws_handler;
use crate::database::devices::Device;
use crate::database::stats::{Stats, TopDomain};
use anyhow::{anyhow, Result};
use axum::extract::{Path, Query, State as AxumState};
use axum::response::IntoResponse;
use axum::routing::{any, get};
use axum::{Json, Router};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{AllowMethods, AllowOrigin, CorsLayer};
use tracing::info;

pub async fn server_root() -> Result<impl IntoResponse, AppError> {
  serve_file(Path("index.html".to_string())).await
}

impl App {
  pub async fn start_dashboard(ctx: Context) -> Result<()> {
    let app = Router::new()
      .route("/", get(server_root))
      .route("/api/devices", get(get_devices_handler).post(create_device_handler))
      .route("/api/devices/{id}", get(get_device_handler).delete(delete_device_handler))
      .route("/api/top", get(top_handler))
      .route("/api/stats", get(stats))
      .route("/api/chart-data", get(chart_data))
      .route("/api/ws", any(ws_handler))
      .route("/{*file}", get(serve_file))
      .layer(
        CorsLayer::new()
          .allow_origin(AllowOrigin::any())
          .allow_private_network(true)
          .allow_methods(AllowMethods::any()),
      )
      .with_state(ctx.clone());

    let addr: SocketAddr = "127.0.0.64:80".parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!("Dashboard backend listening on {addr}");
    Ok(
      axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await?,
    )
  }
}

#[derive(Deserialize)]
struct Limit {
  limit: Option<i64>,
}

async fn top_handler(
  AxumState(ctx): AxumState<Context>,
  Query(limit): Query<Limit>,
) -> Result<Json<Vec<TopDomain>>, AppError> {
  let top = ctx.db().top_blocked(limit.limit).await?;
  Ok(Json(top))
}

#[derive(Deserialize)]
struct StatsQuery {
  since: Option<Duration>,
  until: Option<Duration>,
}

async fn stats(
  AxumState(ctx): AxumState<Context>,
  Query(query): Query<StatsQuery>,
) -> Result<Json<Stats>, AppError> {
  let stats = ctx.db().stats(query.since, query.until).await?;
  Ok(Json(stats))
}

#[derive(Serialize, Deserialize)]
pub struct ChartData {
  pub hour: String,
  pub total: i64,
  pub blocked: i64,
}

async fn chart_data(
  AxumState(ctx): AxumState<Context>,
) -> Result<Json<Vec<ChartData>>, AppError> {
  let rows = ctx.db().stats_by_hour_today().await?;
  let data = rows
    .into_iter()
    .map(|r| ChartData { hour: r.hour, total: r.total, blocked: r.blocked })
    .collect();
  Ok(Json(data))
}

async fn get_devices_handler(
  AxumState(ctx): AxumState<Context>,
) -> Result<Json<Vec<Device>>, AppError> {
  let devices = ctx.db().get_devices().await.map_err(|err| anyhow!("{err}"))?;
  Ok(Json(devices))
}

#[derive(Deserialize)]
struct CreateDevice {
  name: String,
  device_type: String,
}

async fn create_device_handler(
  AxumState(ctx): AxumState<Context>,
  Json(body): Json<CreateDevice>,
) -> Result<Json<Value>, AppError> {
  let id = ctx
    .db()
    .create_device(&body.name, &body.device_type)
    .await
    .map_err(|err| anyhow!("{err}"))?;
  Ok(Json(json!({ "id": id })))
}

async fn get_device_handler(
  AxumState(ctx): AxumState<Context>,
  Path(id): Path<String>,
) -> Result<Json<Device>, AppError> {
  let device = ctx.db().get_device(&id).await.map_err(|e| anyhow!("{e}"))?;
  Ok(Json(device))
}

async fn delete_device_handler(
  AxumState(ctx): AxumState<Context>,
  Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
  ctx.db().delete_device(&id).await.map_err(|e| anyhow!("{e}"))?;
  Ok(Json(json!({ "success": true })))
}