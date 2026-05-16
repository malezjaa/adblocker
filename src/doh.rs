use crate::blocker::{check_block, BlockOrigin};
use crate::server::AppError;
use crate::state::State;
use anyhow::Result;
use axum::body::Bytes;
use axum::extract::{ConnectInfo, Query, State as AxumState};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{routing::post, Router};
use axum_server::tls_rustls::RustlsConfig;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Instant;
use tower_http::trace::TraceLayer;
use tracing::info;

pub async fn setup_doh_server(state: State) -> Result<()> {
  async fn root() -> String {
    "hello".to_string()
  }
  let app = Router::new()
    .route("/", get(root))
    .route("/dns-query", get(doh_get_handler))
    .route("/dns-query", post(doh_handler))
    .layer(TraceLayer::new_for_http())
    .with_state(state);

  let config = RustlsConfig::from_pem_file("certs/cert.pem", "certs/key.pem").await?;

  let addr = SocketAddr::from(([127, 0, 0, 2], 8443));

  info!("DoH server listening on {addr}");
  axum_server::bind_rustls(addr, config)
    .serve(app.into_make_service_with_connect_info::<SocketAddr>())
    .await?;

  Ok(())
}

pub async fn doh_handler(
  AxumState(state): AxumState<State>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  _headers: HeaderMap,
  body: Bytes,
) -> Result<impl IntoResponse, AppError> {
  let start = Instant::now();
  let (blocked, response) =
    check_block(state.clone(), body.to_vec(), BlockOrigin::DoH).await?;
  state.spawn_query_record(
    &response,
    addr,
    blocked,
    BlockOrigin::DoH,
    start.elapsed().as_millis() as i64,
  );

  Ok((
    StatusCode::OK,
    [("content-type", "application/dns-message")],
    Bytes::from(response.to_vec()?),
  ))
}

#[derive(Deserialize)]
pub struct DohQuery {
  dns: String,
}

pub async fn doh_get_handler(
  AxumState(state): AxumState<State>,
  Query(query): Query<DohQuery>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<impl IntoResponse, AppError> {
  let start = Instant::now();
  let bytes = URL_SAFE_NO_PAD.decode(&query.dns)?;

  let (blocked, response) = check_block(state.clone(), bytes, BlockOrigin::DoH).await?;
  state.spawn_query_record(
    &response,
    addr,
    blocked,
    BlockOrigin::DoH,
    start.elapsed().as_millis() as i64,
  );

  Ok((
    StatusCode::OK,
    [("content-type", "application/dns-message")],
    Bytes::from(response.to_vec()?),
  ))
}
