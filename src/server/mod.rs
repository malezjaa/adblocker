pub mod app_error;
pub mod ws;

use crate::application::app::App;
use crate::context::Context;
use crate::db::{Stats, TopDomain};
pub use crate::server::app_error::AppError;
use crate::server::ws::ws_handler;
use anyhow::Result;
use axum::extract::{Query, State as AxumState};
use axum::routing::{any, get};
use axum::{Json, Router};
use chrono::Duration;
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{AllowMethods, AllowOrigin, CorsLayer};
use tracing::info;

impl App {
  pub async fn start_dashboard(ctx: Context) -> Result<()> {
    let app = Router::new()
      .route("/top", get(top_handler))
      .route("/stats", get(stats))
      .route("/", any(ws_handler))
      .layer(
        CorsLayer::new()
          .allow_origin(AllowOrigin::any())
          .allow_methods(AllowMethods::any()),
      )
      .with_state(ctx.clone());

    let addr: SocketAddr = "0.0.0.0:3116".parse()?;
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
