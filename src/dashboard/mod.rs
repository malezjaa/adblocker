pub mod app_error;
pub mod auth;
pub mod endpoints;
pub mod frontend;
pub mod ws;

pub use self::endpoints::stats::QueryLog;
use crate::application::app::App;
use crate::context::Context;
pub use crate::dashboard::app_error::AppError;
use crate::dashboard::auth::{AuthGuard, auth_login, auth_logout, auth_status};
use crate::dashboard::endpoints::devices::{
  create_device_handler, delete_device_handler, get_device_handler, get_devices_handler,
};
use crate::dashboard::endpoints::stats::{
  chart_data, query_logs_handler, stats, top_handler,
};
use crate::dashboard::frontend::serve_file;
use crate::dashboard::ws::ws_handler;
use crate::database::devices::Device;
use crate::database::stats::{Stats, TopDomain};
use anyhow::{Result, anyhow};
use axum::extract::{Path, Query, State as AxumState};
use axum::response::IntoResponse;
use axum::routing::{any, get, post};
use axum::{Json, Router};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{AllowMethods, AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
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
      .route("/api/auth/login", post(auth_login))
      .route("/api/auth/logout", post(auth_logout))
      .route("/api/auth/status", get(auth_status))
      .route("/api/top", get(top_handler))
      .route("/api/stats", get(stats))
      .route("/api/chart-data", get(chart_data))
      .route("/api/query-logs", get(query_logs_handler))
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
