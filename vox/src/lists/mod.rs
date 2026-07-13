pub use vox_lists::{cache, list};

pub mod downloader {
  use std::time::Duration;

  use adblock::FilterSet;
  use anyhow::Result;
  use tokio::{
    spawn,
    time::{interval, sleep},
  };
  use tracing::{error, info};
  use vox_lists::downloader::ListDownloader;

  use crate::{context::Context, engine::EngineMessage};

  async fn _load_blocklists(ctx: &Context) -> Result<(FilterSet, usize)> {
    let blocklists = ctx.blocklists();
    let rules = ctx.config().rules.clone();
    let downloader = ListDownloader::new(ctx.cache_dir(), &blocklists, rules.as_deref());

    let filterset = downloader.load_blocklists().await?;
    ctx.increment_rules_version();

    Ok((filterset, downloader.failed_downloads.len()))
  }

  pub async fn load_blocklists(ctx: &Context) -> Result<FilterSet> {
    let (filterset, failed_downloads) = _load_blocklists(ctx).await?;

    if failed_downloads > 0 {
      info!(
        "failed to download {} {}",
        failed_downloads,
        if failed_downloads == 1 { "list" } else { "lists" }
      );

      let new_ctx = ctx.clone();
      spawn(async move {
        sleep(Duration::from_mins(5)).await;
        if let Err(err) =
          new_ctx.engine_channel().send(EngineMessage::ReloadFilterSet).await
        {
          error!("failed to reload the filter set: {err:?}")
        };
      });
    }

    if ctx.start_blocklist_refresh() {
      let ctx = ctx.clone();
      // Refresh cached lists once per day for the lifetime of this context.
      spawn(async move {
        let mut tick = interval(Duration::from_hours(24));
        tick.tick().await;

        loop {
          tick.tick().await;

          if let Err(err) =
            ctx.engine_channel().send(EngineMessage::ReloadFilterSet).await
          {
            error!("failed to reload the filter set: {err:?}");
            break;
          }
        }
      });
    }

    Ok(filterset)
  }
}
