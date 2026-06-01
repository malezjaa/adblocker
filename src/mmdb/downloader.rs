use anyhow::Result;
use fs_err::create_dir_all;
use std::time::SystemTime;
use time::Duration;
use tokio::fs::{metadata, write};
use tracing::{debug, error};

pub fn download_mmdbs_files() {
  tokio::spawn(async move {
    if let Err(err) = async {
      let home_path = dirs::home_dir().unwrap().join("adb");
      let mmdbs_path = home_path.join("mmdbs");
      create_dir_all(&mmdbs_path)?;

      for file in ["GeoLite2-ASN.mmdb", "GeoLite2-Country.mmdb"] {
        let path = mmdbs_path.join(file);
        let download = if path.exists() {
          let metadata = metadata(&path).await?;
          metadata.modified()?.elapsed()? >= Duration::hours(10)
        } else {
          true
        };

        if download {
          let response = reqwest::get(format!("https://git.io/{file}")).await?;
          response.error_for_status_ref()?;
          write(path, response.bytes().await?).await?;
          debug!("downloaded {file}")
        }
      }

      Ok::<_, anyhow::Error>(())
    }
      .await
    {
      error!("MMDB download failed: {err}");
    }
  });
}
