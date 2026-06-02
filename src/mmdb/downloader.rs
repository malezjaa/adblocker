use anyhow::Result;
use fs_err::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use time::Duration;
use tokio::fs::{metadata, write};
use tracing::{debug, error};

pub static DOWNLOADED_MMDBS: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug)]
pub struct MMDBSPaths {
  pub asn: (String, PathBuf),
  pub country: (String, PathBuf),
}

pub fn download_mmdbs_files() -> MMDBSPaths {
  let mmdbs_path = dirs::home_dir().unwrap().join("adb").join("mmdbs");

  let paths = MMDBSPaths {
    asn: ("GeoLite2-ASN".to_string(), mmdbs_path.join("GeoLite2-ASN.mmdb")),
    country: ("GeoLite2-Country".to_string(), mmdbs_path.join("GeoLite2-Country.mmdb")),
  };

  let new_paths = paths.clone();
  tokio::spawn(async move {
    if let Err(err) = download_mmdbs_inner(mmdbs_path, new_paths).await {
      error!("MMDB download failed: {err}");
    }
  });

  paths
}

async fn download_mmdbs_inner(mmdbs_path: PathBuf, files: MMDBSPaths) -> Result<()> {
  create_dir_all(&mmdbs_path)?;

  let client = reqwest::Client::new();

  for (name, path) in &[&files.asn, &files.country] {
    if !should_download(&path).await? {
      continue;
    }

    let response = client
      .get(format!("https://git.io/{name}.mmdb"))
      .send()
      .await?
      .error_for_status()?;

    write(&path, response.bytes().await?).await?;

    debug!("downloaded {name}");
  }
  DOWNLOADED_MMDBS.store(true, Ordering::Relaxed);

  Ok(())
}

async fn should_download(path: &Path) -> Result<bool> {
  let metadata = match metadata(path).await {
    Ok(metadata) => metadata,
    Err(_) => return Ok(true),
  };

  Ok(metadata.modified()?.elapsed()? >= Duration::days(1))
}
