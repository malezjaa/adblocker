use anyhow::Result;
use axum::{Json, extract::State as AxumState};
use fs_err::tokio::write;
use serde::Deserialize;

use crate::{
  context::Context,
  dashboard::{AppError, auth::AuthGuard},
  lists::{
    cache::load_cache_file,
    list::{LIST_IDS, LISTS, List},
  },
};

pub async fn get_lists_handler(
  _guard: AuthGuard,
  AxumState(ctx): AxumState<Context>,
) -> Result<Json<Vec<List>>, AppError> {
  let cache = load_cache_file(ctx.cache_dir())?;

  let mut lists = LISTS.to_vec();
  lists.iter_mut().for_each(|list| {
    if let Some(cached) = cache.get_by_id(list.id) {
      list.domains = Some(cached.domains);
    }

    list.enabled = Some(ctx.config().blocklists.contains(&list.id.to_string()));
  });

  Ok(Json(lists))
}

#[derive(Deserialize)]
pub struct ToggleListBody {
  pub list_id: String,
}

pub async fn toggle_list(
  _guard: AuthGuard,
  AxumState(ctx): AxumState<Context>,
  Json(body): Json<ToggleListBody>,
) -> Result<Json<()>, AppError> {
  if !LIST_IDS.contains(&body.list_id.as_str()) {
    return Err(AppError::new("Can't toggle a list that doesn't exist".into()));
  }

  let old_config = ctx.config().clone();
  let mut new_config = old_config.clone();

  if new_config.blocklists.contains(&body.list_id) {
    new_config.blocklists.retain(|id| id != &body.list_id);
  } else {
    new_config.blocklists.push(body.list_id);
  }

  write(ctx.config_path(), toml::to_string_pretty(&new_config)?).await?;
  ctx.apply_config_change(old_config, new_config).await?;

  Ok(Json(()))
}
