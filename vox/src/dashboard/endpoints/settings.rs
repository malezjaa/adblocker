use crate::context::Context;
use crate::dashboard::AppError;
use crate::dashboard::auth::AuthGuard;
use anyhow::Result;
use axum::Json;
use axum::extract::State;
use parking_lot::RwLockReadGuard;
use serde::{Deserialize, Serialize};
use vox_shared::config::{Config, UpstreamServer};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
  pub upstreams: Option<Vec<UpstreamServer>>,
  pub dnssec: Option<bool>,
}

fn to_settings(val: RwLockReadGuard<Config>) -> Settings {
  Settings { upstreams: val.upstreams.clone(), dnssec: val.dnssec }
}

pub async fn settings_handler(
  _guard: AuthGuard,
  State(ctx): State<Context>,
) -> Result<Json<Settings>, AppError> {
  let config = ctx.config();
  Ok(Json(to_settings(config)))
}

pub async fn update_settings(
  _guard: AuthGuard,
  State(ctx): State<Context>,
  Json(body): Json<Settings>,
) -> Result<Json<Settings>, AppError> {
  let old_config = ctx.config().clone();
  let mut new_config = old_config.clone();
  new_config.dnssec = body.dnssec;
  new_config.upstreams = body.upstreams;

  fs_err::write(ctx.config_path(), toml::to_string(&new_config)?)?;
  ctx.apply_config_change(old_config, new_config.clone()).await?;
  Ok(Json(Settings {
    upstreams: new_config.upstreams.clone(),
    dnssec: new_config.dnssec,
  }))
}
