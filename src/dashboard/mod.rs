pub mod app_error;
pub mod ws;
pub mod frontend;

use crate::application::app::App;
use crate::context::Context;
pub use crate::dashboard::app_error::AppError;
use crate::dashboard::frontend::serve_file;
use crate::dashboard::ws::ws_handler;
use crate::db::{Stats, TopDomain};
use anyhow::Result;
use axum::extract::{Path, Query, State as AxumState};
use axum::response::IntoResponse;
use axum::routing::{any, get};
use axum::{Json, Router};
use chrono::Duration;
use serde::Deserialize;
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
      .route("/api/top", get(top_handler))
      .route("/api/stats", get(stats))
      .route("/api/ws", any(ws_handler))
      .route("/{*file}", get(serve_file))
      .layer(
        CorsLayer::new()
          .allow_origin(AllowOrigin::any())
          .allow_methods(AllowMethods::any()),
      )
      .with_state(ctx.clone());

    let addr: SocketAddr = "127.0.0.64:80".parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!("Dashboard backend listening on {addr}");
    Ok(axum::serve(listener, app).await?)
  }
}

#[derive(Deserialize)]
struct Limit {
  limit: i64,
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
