use adblock::lists::ParseOptions;
use adblock::FilterSet;
use anyhow::Result;
use futures::future::join_all;
use std::time::Instant;
use tracing::{debug, info};

pub async fn load_blocklists(blocklists: Vec<String>) -> Result<FilterSet> {
  let mut filterset = FilterSet::new(false);

  let futures = blocklists.iter().map(|blocklist| async move {
    let fetch_start = Instant::now();
    let body = reqwest::get(blocklist).await?.text().await?;
    debug!(%blocklist, elapsed_ms = fetch_start.elapsed().as_millis(), "fetched");

    let parse_start = Instant::now();
    let rules = body
      .lines().map(|l| l.to_string()).collect::<Vec<_>>();
    debug!(%blocklist, elapsed_ms = parse_start.elapsed().as_millis(), rule_count = rules.len(), "parsed");

    Ok::<_, anyhow::Error>(rules.clone())
  });
  let results = join_all(futures).await;

  for result in results {
    filterset.add_filters(result?, ParseOptions::default());
  }

  Ok(filterset)
}
