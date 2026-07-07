use crate::cache::{CacheFile, load_cache_file};
use crate::list::{LISTS, List};
use adblock::FilterSet;
use adblock::lists::ParseOptions;
use anyhow::{Result, bail};
use chrono::Duration;
use fs_err::{read, write};
use futures::future::join_all;
use reqwest::{Client, StatusCode};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{debug, error, info};
use vox_shared::config::rules::Rule;

pub async fn download_blocklist(
  list: &List,
  cache_dir: &Path,
  cached_etag: Option<String>,
  is_fresh: bool,
) -> Result<(String, Vec<String>, Option<String>)> {
  let cache_file = cache_dir.join(CacheFile::id_hash(list.id));

  if is_fresh && cache_file.exists() {
    return read_rules(list.id, &cache_file, None);
  }

  let client = Client::new();
  let mut req = client.get(list.url.to_string());
  if let Some(etag) = cached_etag {
    req = req.header("If-None-Match", etag);
  }

  let resp = req.send().await?;
  let new_etag =
    resp.headers().get("etag").and_then(|v| v.to_str().ok()).map(str::to_owned);
  let status = resp.status();

  if status == StatusCode::NOT_MODIFIED {
    return read_rules(list.id, &cache_file, new_etag);
  }

  if status == StatusCode::NOT_FOUND {
    bail!("Couldn't download {} list, because it doesn't exist", list.id)
  }

  let body = resp.text().await?;

  if status.is_server_error() || status.is_client_error() {
    bail!("failed to download {} list: {body}", list.id)
  }

  let tmp = cache_file.with_extension("tmp");
  write(&tmp, &body)?;
  fs_err::rename(&tmp, &cache_file)?;
  info!(name = %list.name, "downloaded blocklist");

  let rules = body.lines().map(|l| l.to_string()).collect::<Vec<_>>();
  Ok((list.id.to_string(), rules, new_etag))
}

pub async fn load_blocklists(
  cache_dir: &Path,
  enabled_ids: &[String],
  custom_rules: Option<&[Rule]>,
) -> Result<FilterSet> {
  let start = Instant::now();
  let mut filterset = FilterSet::new(false);
  let mut cache = load_cache_file(cache_dir)?;

  let mut total = 0;
  if let Some(rules) = custom_rules {
    total += rules.len();
    let rules = rules.iter().map(Rule::adblock_rule).collect::<Vec<_>>();
    filterset.add_filters(&rules, Default::default());
    info!(
      "loaded {} custom block {}",
      rules.len(),
      if rules.len() == 1 { "rule" } else { "rules" }
    );
  }

  let futures = LISTS.iter().map(|list| {
    let is_fresh = cache.is_fresh(list.id, Duration::hours(24));
    let cached_etag = cache.get_by_id(list.id).and_then(|e| e.etag.clone());
    async move { download_blocklist(list, cache_dir, cached_etag, is_fresh).await }
  });

  let results: Vec<Result<_>> = join_all(futures).await;

  for (id, rules, etag) in results.iter().flatten() {
    cache.insert(id, etag.clone(), rules.len());
  }

  write(cache_dir.join("cache.toml"), toml::to_string(&cache)?)?;

  let configured_ids: HashSet<&str> = enabled_ids.iter().map(|s| s.as_str()).collect();

  for result in results {
    match result {
      Ok((id, rules, _)) => {
        if configured_ids.contains(id.as_str()) {
          total += rules.len();
          filterset.add_filters(&rules, ParseOptions::default());
          info!(%id, "loaded blocklist into filterset");
        }
      }
      Err(e) => {
        error!("failed to download/read blocklist: {e:#}");
      }
    }
  }

  debug!("loaded {} rules in total", total);
  info!("loaded lists in {:.2?}", start.elapsed());

  Ok(filterset)
}

fn read_rules(
  id: &str,
  cache_file: &PathBuf,
  etag: Option<String>,
) -> Result<(String, Vec<String>, Option<String>)> {
  let content = read(cache_file)?;
  let rules =
    String::from_utf8(content)?.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
  Ok((id.to_owned(), rules, etag))
}
