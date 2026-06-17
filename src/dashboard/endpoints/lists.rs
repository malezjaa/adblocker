use crate::context::Context;
use crate::dashboard::AppError;
use crate::dashboard::auth::AuthGuard;
use crate::lists::cache::load_cache_file;
use crate::lists::list::{LISTS_IDS, List, default_lists};
use anyhow::Result;
use axum::Json;
use axum::extract::State as AxumState;
use fs_err::tokio::write;
use serde::Deserialize;

pub async fn get_lists_handler(
  _guard: AuthGuard,
  AxumState(ctx): AxumState<Context>,
) -> Result<Json<Vec<List>>, AppError> {
  let cache = load_cache_file(ctx.cache_dir())?;

  let mut lists = default_lists();
  lists.iter_mut().for_each(|list| {
    if let Some(cached) = cache.get_by_id(&list.id) {
      list.domains = Some(cached.domains.clone());
    }

    list.enabled = Some(ctx.config().blocklists.contains(&list.id));
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
  if !LISTS_IDS.contains(&body.list_id.as_str()) {
    return Err(AppError::new("Can't toggle a list that doesn't exist".into()));
  }

  let toml = {
    let mut config = ctx.0.config.write();

    if config.blocklists.contains(&body.list_id) {
      config.blocklists.retain(|id| id != &body.list_id);
    } else {
      config.blocklists.push(body.list_id);
    }

    toml::to_string(&*config)?
  };

  write(ctx.config_path(), toml).await?;

  Ok(Json(()))
}
