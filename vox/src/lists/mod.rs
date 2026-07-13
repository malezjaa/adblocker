pub use vox_lists::{cache, list};

pub mod downloader {
  use adblock::FilterSet;
  use anyhow::Result;
  pub use vox_lists::downloader::download_blocklist;

  use crate::context::Context;

  pub async fn load_blocklists(ctx: &Context) -> Result<FilterSet> {
    let blocklists = ctx.blocklists();
    let rules = ctx.config().rules.clone();
    let filterset = vox_lists::downloader::load_blocklists(
      ctx.cache_dir(),
      &blocklists,
      rules.as_deref(),
    )
    .await?;

    ctx.increment_rules_version();

    Ok(filterset)
  }
}
