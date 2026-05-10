pub mod app_error;

pub use crate::server::app_error::AppError;
use crate::state::{State, TopDomain};
use anyhow::Result;
use axum::extract::{Query, State as AxumState};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use tokio::net::TcpListener;

pub async fn setup_server(state: State) -> Result<()> {
  let app = Router::new().route("/top", get(top_handler)).with_state(state);

  let listener = TcpListener::bind("0.0.0.0:3000").await?;
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
