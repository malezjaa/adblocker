use crate::application::app::App;
use crate::context::Context;
use crate::dashboard::AppError;
use anyhow::Result;
use axum::body::Bytes;
use axum::extract::{ConnectInfo, Path, Query, State as AxumState};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use vox_dns::block_origin::BlockOrigin;

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
      .route("/dns-query", get(Self::doh_get_handler).post(Self::doh_handler))
      .route(
        "/dns-query/{device}",
        get(Self::doh_get_device_handler).post(Self::doh_device_handler),
      )
      .layer(TraceLayer::new_for_http())
      .with_state(ctx.clone());

    let listener = TcpListener::bind(ctx.doh_socket()).await?;
    info!("DoH server listening on {}", ctx.doh_socket());

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
      .await?;

    Ok(())
  }

  async fn handle_doh_request(
    ctx: Context,
    addr: SocketAddr,
    bytes: Vec<u8>,
    device: Option<String>,
  ) -> Result<impl IntoResponse, AppError> {
    let response = ctx.query_dns(bytes, BlockOrigin::doh(), addr, device).await?;
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
    Self::handle_doh_request(ctx, addr, bytes, None).await
  }

  pub async fn doh_handler(
    AxumState(ctx): AxumState<Context>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    body: Bytes,
  ) -> Result<impl IntoResponse, AppError> {
    Self::handle_doh_request(ctx, addr, body.to_vec(), None).await
  }

  pub async fn doh_get_device_handler(
    AxumState(ctx): AxumState<Context>,
    Path(device): Path<String>,
    Query(query): Query<DohQuery>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
  ) -> Result<impl IntoResponse, AppError> {
    let bytes = URL_SAFE_NO_PAD.decode(&query.dns)?;
    Self::handle_doh_request(ctx, addr, bytes, Some(device)).await
  }

  pub async fn doh_device_handler(
    AxumState(ctx): AxumState<Context>,
    Path(device): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    body: Bytes,
  ) -> Result<impl IntoResponse, AppError> {
    Self::handle_doh_request(ctx, addr, body.to_vec(), Some(device)).await
  }
}
