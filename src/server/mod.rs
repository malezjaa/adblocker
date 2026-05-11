pub mod app_error;
pub mod ws;

use crate::db::{Stats, TopDomain};
pub use crate::server::app_error::AppError;
use crate::server::ws::ws_handler;
use crate::state::State;
use anyhow::Result;
use axum::extract::{Query, State as AxumState};
use axum::response::IntoResponse;
use axum::routing::{any, get};
use axum::{Json, Router};
use chrono::Duration;
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::cors::{AllowMethods, AllowOrigin, CorsLayer};

pub async fn setup_server(state: State) -> Result<()> {
  let app = Router::new()
    .route("/top", get(top_handler))
    .route("/stats", get(stats))
    .route("/", any(ws_handler))
    .layer(
      CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods(AllowMethods::any()),
    )
    .with_state(state);

  let listener = TcpListener::bind("0.0.0.0:3116").await?;
  Ok(axum::serve(listener, app).await?)
}

#[derive(Deserialize)]
struct Limit {
  limit: i64,
}

async fn top_handler(
  AxumState(state): AxumState<State>,
  Query(limit): Query<Limit>,
) -> Result<Json<Vec<TopDomain>>, AppError> {
  let top = state.top_blocked(limit.limit).await?;
  Ok(Json(top))
}

#[derive(Deserialize)]
struct StatsQuery {
  since: Option<Duration>,
  until: Option<Duration>,
}

async fn stats(
  AxumState(state): AxumState<State>,
  Query(query): Query<StatsQuery>,
) -> Result<Json<Stats>, AppError> {
  let stats = state.stats(query.since, query.until).await?;
  Ok(Json(stats))
}
