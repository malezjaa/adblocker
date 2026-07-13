use anyhow::Result;
use axum::{
  Json,
  extract::{Path, State},
};
use serde::Serialize;
use vox_shared::config::{Config, rewrite::Rewrite};

use crate::{
  app_error,
  context::Context,
  dashboard::{AppError, auth::AuthGuard},
};

#[derive(Serialize)]
pub struct RewriteEntry {
  pub index: usize,
  pub rewrite: Rewrite,
}

fn rewrite_entries(config: &Config) -> Vec<RewriteEntry> {
  config
    .rewrites
    .clone()
    .unwrap_or_default()
    .into_iter()
    .enumerate()
    .map(|(index, rewrite)| RewriteEntry { index, rewrite })
    .collect()
}

async fn save_rewrites(
  ctx: &Context,
  old_config: Config,
  rewrites: Vec<Rewrite>,
) -> Result<()> {
  let mut new_config = old_config.clone();
  new_config.rewrites = if rewrites.is_empty() { None } else { Some(rewrites) };
  new_config.compile_regexes()?;

  fs_err::write(ctx.config_path(), toml::to_string_pretty(&new_config)?)?;
  ctx.apply_config_change(old_config, new_config).await?;

  Ok(())
}

pub async fn get_rewrites(
  _guard: AuthGuard,
  State(ctx): State<Context>,
) -> Result<Json<Vec<RewriteEntry>>, AppError> {
  let config = ctx.config();
  Ok(Json(rewrite_entries(&config)))
}

pub async fn create_rewrite(
  _guard: AuthGuard,
  State(ctx): State<Context>,
  Json(body): Json<Rewrite>,
) -> Result<Json<Vec<RewriteEntry>>, AppError> {
  let old_config = ctx.config().clone();
  let mut rewrites = old_config.rewrites.clone().unwrap_or_default();

  rewrites.push(body);
  save_rewrites(&ctx, old_config, rewrites).await?;

  let config = ctx.config();
  Ok(Json(rewrite_entries(&config)))
}

pub async fn update_rewrite(
  _guard: AuthGuard,
  State(ctx): State<Context>,
  Path(index): Path<usize>,
  Json(body): Json<Rewrite>,
) -> Result<Json<Vec<RewriteEntry>>, AppError> {
  let old_config = ctx.config().clone();
  let mut rewrites = old_config.rewrites.clone().unwrap_or_default();

  let Some(rewrite) = rewrites.get_mut(index) else {
    app_error!("Rewrite with index {} not found", index);
  };

  *rewrite = body;
  save_rewrites(&ctx, old_config, rewrites).await?;

  let config = ctx.config();
  Ok(Json(rewrite_entries(&config)))
}

pub async fn delete_rewrite(
  _guard: AuthGuard,
  State(ctx): State<Context>,
  Path(index): Path<usize>,
) -> Result<Json<Vec<RewriteEntry>>, AppError> {
  let old_config = ctx.config().clone();
  let mut rewrites = old_config.rewrites.clone().unwrap_or_default();

  if index >= rewrites.len() {
    app_error!("Rewrite with index {} not found", index);
  }

  rewrites.remove(index);
  save_rewrites(&ctx, old_config, rewrites).await?;

  let config = ctx.config();
  Ok(Json(rewrite_entries(&config)))
}
