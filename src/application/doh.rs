use crate::application::app::App;
use crate::context::Context;
use crate::dashboard::AppError;
use crate::engine::{BlockOrigin, process_message};
use anyhow::Result;
use axum::body::Bytes;
use axum::extract::{ConnectInfo, Query, State as AxumState};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Router, routing::post};
use axum_server::tls_rustls::RustlsConfig;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Instant;
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Deserialize)]
pub struct DohQuery {
  dns: String,
}

impl App {
  pub async fn start_doh(ctx: Context) -> Result<()> {
    async fn root() -> String {
      "DoH dashboard".to_string()
    }
    let app = Router::new()
      .route("/", get(root))
      .route("/dns-query", get(Self::doh_get_handler))
      .route("/dns-query", post(Self::doh_handler))
      .layer(TraceLayer::new_for_http())
      .with_state(ctx.clone());

    let config = RustlsConfig::from_config(ctx.server_config());
    let addr = SocketAddr::from(([127, 0, 0, 2], 8443));

    info!("DoH dashboard listening on {addr}");
    axum_server::bind_rustls(addr, config)
      .serve(app.into_make_service_with_connect_info::<SocketAddr>())
      .await?;

    Ok(())
  }

  async fn handle_doh_request(
    ctx: Context,
    addr: SocketAddr,
    bytes: Vec<u8>,
  ) -> Result<impl IntoResponse, AppError> {
    let start = Instant::now();
    let (blocked, response) =
      process_message(ctx.clone(), bytes, BlockOrigin::DoH).await?;
    ctx.db().spawn_query_record(
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

  pub async fn doh_get_handler(
    AxumState(ctx): AxumState<Context>,
    Query(query): Query<DohQuery>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
  ) -> Result<impl IntoResponse, AppError> {
    let bytes = URL_SAFE_NO_PAD.decode(&query.dns)?;
    Self::handle_doh_request(ctx, addr, bytes).await
  }

  pub async fn doh_handler(
    AxumState(ctx): AxumState<Context>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    body: Bytes,
  ) -> Result<impl IntoResponse, AppError> {
    Self::handle_doh_request(ctx, addr, body.to_vec()).await
  }
}
