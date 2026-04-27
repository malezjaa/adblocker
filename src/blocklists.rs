use adblock::lists::ParseOptions;
use adblock::FilterSet;
use anyhow::Result;
use tracing::{debug, info};

pub async fn load_blocklists(blocklists: Vec<String>) -> Result<FilterSet> {
  let mut filterset = FilterSet::new(false);

  // TODO: cache this
  for blocklist in blocklists {
    let request = reqwest::get(&blocklist).await?;
    info!(status = %request.status(), %blocklist, "blocklist request");

    let body = request.text().await?;
    let rules = body.lines().collect::<Vec<_>>();

    filterset.add_filters(rules, ParseOptions::default());
  }

  Ok(filterset)
}