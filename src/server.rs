use crate::state::{State, TopDomain};
use anyhow::Result;
use axum::extract::{Query, State as AxumState};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({ "error": self.0.to_string() })),
    )
      .into_response()
  }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
  fn from(e: E) -> Self {
    Self(e.into())
  }
}

pub async fn setup_server(state: State) -> Result<()> {
  let app = Router::new()
    .route("/top", get(top_handler))
    .with_state(state);

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