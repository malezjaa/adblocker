use crate::config::{Config, UpstreamServer};
use crate::context::Context;
use crate::dashboard::AppError;
use crate::dashboard::auth::AuthGuard;
use anyhow::Result;
use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
  pub upstreams: Option<Vec<UpstreamServer>>,
  pub dnssec: Option<bool>,
}

impl Config {
  fn as_settings(&self) -> Settings {
    Settings { upstreams: self.upstreams.clone(), dnssec: self.dnssec }
  }
}

pub async fn settings_handler(
  _guard: AuthGuard,
  State(ctx): State<Context>,
) -> Result<Json<Settings>, AppError> {
  let config = ctx.config();
  Ok(Json(config.as_settings()))
}

pub async fn update_settings(
  _guard: AuthGuard,
  State(ctx): State<Context>,
  Json(body): Json<Settings>,
) -> Result<Json<Settings>, AppError> {
  let mut config = ctx.config().clone();
  config.dnssec = body.dnssec;
  config.upstreams = body.upstreams;

  fs_err::write(ctx.config_path(), toml::to_string(&config)?)?;
  Ok(Json(config.as_settings()))
}
