use crate::config::Config;
use crate::context::Context;
use anyhow::Result;
use futures::future::pending;
use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, new_debouncer};
use std::time::Duration;
use tokio::spawn;
use tracing::{debug, error, info, warn};

impl Config {
  pub fn spawn_config_watcher(ctx: Context) -> Result<()> {
    spawn(async move {
      if let Err(err) = async {
        let config_path = ctx.0.config_path.clone();
        let ctx_clone = ctx.clone();

        let mut debouncer = new_debouncer(
          Duration::from_secs(1),
          None,
          move |result: DebounceEventResult| match result {
            Ok(events) => {
              for event in events {
                match event.kind {
                  EventKind::Modify(_) | EventKind::Create(_) => {
                    match Config::from_file(config_path.clone()) {
                      Ok(mut config) => {
                        config.compile_regexes().unwrap();
                        *ctx_clone.0.config.write() = config;
                        info!("reloaded config");
                      }
                      Err(err) => {
                        error!("failed to reload config: {err}");
                      }
                    }
                  }

                  EventKind::Remove(_) => {
                    warn!("config was removed");
                  }

                  _ => {}
                }
              }
            }

            Err(errors) => {
              errors.iter().for_each(|error| error!("{error:?}"));
            }
          },
        )?;

        debouncer.watch(&ctx.0.config_path, RecursiveMode::NonRecursive)?;
        debug!("started config watcher");
        pending::<()>().await;

        Ok::<(), anyhow::Error>(())
      }
      .await
      {
        error!("config watcher failed: {err}");
      }
    });

    Ok(())
  }
}
