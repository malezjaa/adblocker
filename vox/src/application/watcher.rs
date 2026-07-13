use crate::context::Context;
use crate::dns::resolver::create_hickory_resolver;
use crate::engine::EngineMessage;
use anyhow::Result;
use futures::future::pending;
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, new_debouncer};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::spawn;
use tokio::sync::mpsc;
use tracing::{debug, error, warn};
use vox_shared::config::Config;

impl Context {
  pub fn spawn_config_watcher(&self) -> Result<()> {
    let ctx = self.clone();
    spawn(async move {
      if let Err(err) = ctx.run_watcher().await {
        error!("config watcher failed: {err}");
      }
    });

    Ok(())
  }

  async fn run_watcher(&self) -> Result<()> {
    let config_path = self.config_path().to_path_buf();

    let (tx, mut rx) = mpsc::channel::<DebounceEventResult>(64);

    let mut debouncer =
      new_debouncer(Duration::from_secs(1), None, move |result: DebounceEventResult| {
        let _ = tx.blocking_send(result);
      })?;

    debouncer.watch(&config_path, RecursiveMode::NonRecursive)?;
    debug!("started config watcher");

    let ctx_clone = self.clone();
    let config_path_clone = config_path.clone();

    let processor = spawn(async move {
      while let Some(result) = rx.recv().await {
        handle_debounce_result(result, &config_path_clone, &ctx_clone).await;
      }
    });

    let _ = tokio::join!(pending::<()>(), processor);

    Ok(())
  }
}

async fn handle_debounce_result(
  result: DebounceEventResult,
  config_path: &PathBuf,
  ctx: &Context,
) {
  let events = match result {
    Ok(events) => events,
    Err(errors) => {
      for error in &errors {
        error!("{error:?}");
      }
      return;
    }
  };

  let mut should_reload = false;
  let mut removed = false;

  for event in &events {
    match event.kind {
      EventKind::Modify(_) | EventKind::Create(_) => should_reload = true,
      EventKind::Remove(_) => removed = true,
      _ => {}
    }
  }

  if removed {
    warn!("config was removed");
  }

  if should_reload {
    reload_config(config_path, ctx).await;
  }
}

async fn reload_config(config_path: &Path, ctx: &Context) {
  let config = match Config::from_file(config_path) {
    Ok(config) => config,
    Err(err) => {
      error!("failed to reload config: {err}");
      return;
    }
  };

  match create_hickory_resolver(&config) {
    Ok(resolver) => ctx.update_resolver(resolver),
    Err(err) => error!("failed to create new hickory resolver: {:?}", err),
  }

  let old_config = ctx.config().clone();
  if let Err(err) = ctx.apply_config_change(old_config, config).await {
    error!("failed to update config: {err:?}")
  };
}

impl Context {
  pub async fn apply_config_change(
    &self,
    old_config: Config,
    new_config: Config,
  ) -> Result<()> {
    let mut blocklists = old_config.blocklists.clone();
    let mut new_blocklists = new_config.blocklists.clone();
    blocklists.sort();
    new_blocklists.sort();

    let mut rules = old_config.rules.clone().unwrap_or_default();
    let mut new_rules = new_config.rules.clone().unwrap_or_default();

    rules.sort_by(|a, b| a.domain.cmp(&b.domain));
    new_rules.sort_by(|a, b| a.domain.cmp(&b.domain));

    let filters_changed = blocklists != new_blocklists;
    let rules_changed = rules != new_rules;

    self.update_config(new_config.clone());

    if filters_changed || rules_changed {
      self.tx().send(EngineMessage::ReloadFilterSet).await?;
    }

    Ok(())
  }
}
