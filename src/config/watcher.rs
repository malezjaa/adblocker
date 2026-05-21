use crate::config::Config;
use crate::context::Context;
use anyhow::Result;
use notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use std::time::Duration;

impl Config {
  pub fn spawn_config_watcher(ctx: Context) -> Result<()> {
    let mut debouncer = new_debouncer(Duration::from_secs(2), None, |result: DebounceEventResult| {
      match result {
        Ok(events) => events.iter().for_each(|event| println!("{event:?}")),
        Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
      }
    })?;

    debouncer.watch(ctx.0.config_path.clone(), RecursiveMode::NonRecursive)?;
    Ok(())
  }
}
