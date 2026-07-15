use std::{
  collections::HashSet,
  path::{Path, PathBuf},
  time::Instant,
};

use adblock::{FilterSet, lists::ParseOptions};
use anyhow::{Result, bail};
use chrono::Duration;
use dashmap::DashSet;
use fs_err::{read, write};
use futures::future::join_all;
use reqwest::{Client, ClientBuilder, StatusCode};
use tracing::{debug, error, info};
use vox_shared::config::rules::Rule;

use crate::{
  cache::{CacheFile, load_cache_file},
  list::{LISTS, List},
};

struct DownloadedBlocklist {
  id: String,
  contents: String,
  rule_count: usize,
  etag: Option<String>,
}

impl DownloadedBlocklist {
  fn new(id: &str, contents: String, etag: Option<String>) -> Self {
    let rule_count = contents.lines().count();

    Self { id: id.to_owned(), contents, rule_count, etag }
  }
}

pub struct ListDownloader<'a> {
  cache_dir: &'a Path,
  enabled_ids: &'a [String],
  custom_rules: Option<&'a [Rule]>,
  client: Client,
  pub failed_downloads: DashSet<String>,
}

impl<'a> ListDownloader<'a> {
  pub fn new(
    cache_dir: &'a Path,
    enabled_ids: &'a [String],
    custom_rules: Option<&'a [Rule]>,
  ) -> Result<Self> {
    Ok(Self {
      cache_dir,
      enabled_ids,
      custom_rules,
      failed_downloads: DashSet::new(),
      client: ClientBuilder::new().zstd(true).brotli(true).gzip(true).build()?,
    })
  }

  async fn download_blocklist(
    &self,
    list: &List,
    cached_etag: Option<String>,
    is_fresh: bool,
  ) -> Result<DownloadedBlocklist> {
    let cache_file = self.cache_dir.join(CacheFile::id_hash(list.id));

    if is_fresh && cache_file.exists() {
      match Self::read_rules(list.id, &cache_file, None) {
        Ok(rules) => return Ok(rules),
        Err(err) => {
          error!(%err, id = list.id, "failed to read cached blocklist; downloading a replacement");
        }
      }
    }

    let mut req = self.client.get(list.url.to_string());
    if let Some(etag) = cached_etag {
      req = req.header("If-None-Match", etag);
    }

    let resp = req.send().await.map_err(|err| {
      self.failed_downloads.insert(list.id.to_owned());
      err
    })?;

    let new_etag =
      resp.headers().get("etag").and_then(|v| v.to_str().ok()).map(str::to_owned);
    let status = resp.status();

    if status == StatusCode::NOT_MODIFIED {
      return Self::read_rules(list.id, &cache_file, new_etag).map_err(|err| {
        self.failed_downloads.insert(list.id.to_owned());
        err
      });
    }

    // We won't try to retry on 404, because it most likely means that the link got
    // broken and needs to get updated
    if status == StatusCode::NOT_FOUND {
      bail!("Couldn't download {} list, because it doesn't exist", list.id)
    }

    let body = resp.text().await.map_err(|err| {
      self.failed_downloads.insert(list.id.to_owned());
      err
    })?;
    if status.is_server_error() || status.is_client_error() {
      self.failed_downloads.insert(list.id.to_owned());
      bail!("failed to download {} list: {body}", list.id)
    }

    let tmp = cache_file.with_extension("tmp");
    write(&tmp, &body).map_err(|err| {
      self.failed_downloads.insert(list.id.to_owned());
      err
    })?;
    fs_err::rename(&tmp, &cache_file).map_err(|err| {
      self.failed_downloads.insert(list.id.to_owned());
      err
    })?;
    info!(name = %list.name, "downloaded blocklist");

    Ok(DownloadedBlocklist::new(list.id, body, new_etag))
  }

  pub async fn load_blocklists(&self) -> Result<FilterSet> {
    let start = Instant::now();
    let mut filterset = FilterSet::new(false);
    let mut cache = load_cache_file(self.cache_dir)?;

    let mut total = 0;
    if let Some(rules) = self.custom_rules {
      total += rules.len();
      let rules = rules.iter().map(Rule::adblock_rule).collect::<Vec<_>>();
      filterset.add_filter_list(rules.join("\n"), Default::default());
      info!(
        "loaded {} custom block {}",
        rules.len(),
        if rules.len() == 1 { "rule" } else { "rules" }
      );
    }

    let configured_ids: HashSet<&str> =
      self.enabled_ids.iter().map(|s| s.as_str()).collect();

    let futures =
      LISTS.iter().filter(|list| configured_ids.contains(list.id)).map(|list| {
        let is_fresh = cache.is_fresh(list.id, Duration::hours(24));
        let cached_etag = cache.get_by_id(list.id).and_then(|e| e.etag.clone());
        async move { self.download_blocklist(list, cached_etag, is_fresh).await }
      });

    let results: Vec<Result<_>> = join_all(futures).await;

    for list in results.iter().flatten() {
      cache.insert(&list.id, list.etag.clone(), list.rule_count);
    }

    write(self.cache_dir.join("cache.toml"), toml::to_string_pretty(&cache)?)?;

    for result in results {
      match result {
        Ok(list) => {
          if configured_ids.contains(list.id.as_str()) {
            total += list.rule_count;
            filterset.add_filter_list(list.contents, ParseOptions::default());
            info!(id = %list.id, "loaded blocklist into filterset");
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
  ) -> Result<DownloadedBlocklist> {
    let contents = String::from_utf8(read(cache_file)?)?;
    Ok(DownloadedBlocklist::new(id, contents, etag))
  }
}
