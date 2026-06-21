use crate::config::Config;
use crate::context::Context;
use crate::dns::resolver::create_hickory_resolver;
use crate::engine::EngineMessage;
use anyhow::Result;
use futures::future::pending;
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, new_debouncer};
use std::path::PathBuf;
use std::time::Duration;
use tokio::spawn;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

impl Config {
  pub fn spawn_config_watcher(ctx: Context) -> Result<()> {
    spawn(async move {
      if let Err(err) = run_watcher(ctx).await {
        error!("config watcher failed: {err}");
      }
    });

    Ok(())
  }
}

async fn run_watcher(ctx: Context) -> Result<()> {
  let config_path = ctx.0.config_path.clone();

  let (tx, mut rx) = mpsc::channel::<DebounceEventResult>(64);

  let mut debouncer =
    new_debouncer(Duration::from_secs(1), None, move |result: DebounceEventResult| {
      let _ = tx.blocking_send(result);
    })?;

  debouncer.watch(&config_path, RecursiveMode::NonRecursive)?;
  debug!("started config watcher");

  let ctx_clone = ctx.clone();
  let config_path_clone = config_path.clone();

  let processor = spawn(async move {
    while let Some(result) = rx.recv().await {
      handle_debounce_result(result, &config_path_clone, &ctx_clone).await;
    }
  });

  let _ = tokio::join!(pending::<()>(), processor);

  Ok(())
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

async fn reload_config(config_path: &PathBuf, ctx: &Context) {
  let mut config = match Config::from_file(config_path.clone()) {
    Ok(config) => config,
    Err(err) => {
      error!("failed to reload config: {err}");
      return;
    }
  };

  if let Err(err) = config.compile_regexes() {
    error!("failed to compile regexes in reloaded config: {err}");
    return;
  }

  match create_hickory_resolver(&config) {
    Ok(resolver) => ctx.update_resolver(resolver),
    Err(err) => error!("failed to create new hickory resolver: {:?}", err),
  }

  *ctx.0.config.write() = config;
  info!("reloaded config");

  if let Err(err) = ctx.tx().send(EngineMessage::ReloadFilterSet).await {
    error!("failed to notify engine of config reload: {err}");
  }
}
