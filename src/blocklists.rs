use crate::cache::CacheFile;
use adblock::lists::ParseOptions;
use adblock::FilterSet;
use anyhow::Result;
use fs_err::{create_dir, create_dir_all, read, write};
use futures::future::join_all;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use tracing::{debug, info};

pub fn load_cache_file(cache_dir: &Path) -> Result<CacheFile> {
  let path = cache_dir.join("cache.toml");
  info!(path = path.display().to_string(), "loading config");

  if !path.exists() {
    let cache = CacheFile::default();
    if let Some(parent) = path.parent() {
      create_dir_all(parent)?;
    }

    write(path, toml::to_string(&cache)?)?;
    return Ok(cache);
  }

  let content = read(path)?;
  Ok(toml::from_slice(&content)?)
}

pub async fn load_blocklists(blocklists: Vec<String>, cache_dir: &Path) -> Result<FilterSet> {
  let mut filterset = FilterSet::new(false);
  let mut cache = load_cache_file(cache_dir)?;
  let max_age = chrono::Duration::hours(24);

  let futures = blocklists.iter().map(|blocklist| {
    let cache_file = cache_dir.join(CacheFile::url_hash(blocklist));
    let is_fresh = cache.is_fresh(blocklist, max_age);
    let cached_etag = cache.get_by_url(blocklist).and_then(|e| e.etag.clone());

    async move {
      if is_fresh && cache_file.exists() {
        let content = read(&cache_file)?;
        let rules = String::from_utf8(content)?
          .lines()
          .map(|l| l.to_string())
          .collect::<Vec<_>>();
        info!(%blocklist, "using cached");
        return Ok::<_, anyhow::Error>((blocklist.clone(), rules, None));
      }

      let client = Client::new();
      let mut req = client.get(blocklist);
      if let Some(etag) = cached_etag {
        req = req.header("If-None-Match", etag);
      }

      let resp = req.send().await?;
      let new_etag = resp
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

      if resp.status() == StatusCode::NOT_MODIFIED {
        info!(%blocklist, "304 not modified, using cache");
        let content = read(&cache_file)?;
        let rules = String::from_utf8(content)?
          .lines()
          .map(|l| l.to_string())
          .collect::<Vec<_>>();
        return Ok((blocklist.clone(), rules, new_etag));
      }

      let body = resp.text().await?;
      let tmp = cache_file.with_extension("tmp");
      write(&tmp, &body)?;
      fs_err::rename(&tmp, &cache_file)?;
      info!(%blocklist, "using blocklist: ");

      let rules = body.lines().map(|l| l.to_string()).collect::<Vec<_>>();
      Ok((blocklist.clone(), rules, new_etag))
    }
  });

  let results = join_all(futures).await;

  for result in results {
    let (url, rules, etag) = result?;
    cache.insert(&url, etag);
    filterset.add_filters(rules, ParseOptions::default());
  }

  write(cache_dir.join("cache.toml"), toml::to_string(&cache)?)?;

  Ok(filterset)
}