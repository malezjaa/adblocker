use crate::context::Context;
use crate::dashboard::AppError;
use crate::dashboard::auth::AuthGuard;
use crate::dns::resolver::create_hickory_resolver;
use anyhow::Result;
use axum::Json;
use axum::extract::State;
use vox_shared::config::Config;

pub async fn settings_handler(
  _guard: AuthGuard,
  State(ctx): State<Context>,
) -> Result<Json<Config>, AppError> {
  let config = ctx.config();
  Ok(Json(config.clone()))
}

pub async fn update_settings(
  _guard: AuthGuard,
  State(ctx): State<Context>,
  Json(mut new_config): Json<Config>,
) -> Result<Json<Config>, AppError> {
  let old_config = ctx.config().clone();

  new_config.compile_regexes()?;
  new_config.validate_rules();
  let resolver = create_hickory_resolver(&new_config)?;

  fs_err::write(ctx.config_path(), toml::to_string(&new_config)?)?;
  ctx.update_resolver(resolver);
  ctx.apply_config_change(old_config, new_config.clone()).await?;

  Ok(Json(new_config))
}
