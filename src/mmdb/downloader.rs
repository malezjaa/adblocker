use anyhow::Result;
use fs_err::create_dir_all;
use tokio::fs::write;
use tracing::debug;

pub async fn download_mmdbs_files() -> Result<()> {
  let home_path = dirs::home_dir().unwrap().join("adb");
  let mmdbs_path = home_path.join("mmdbs");
  create_dir_all(&mmdbs_path)?;

  for file in ["GeoLite2-ASN.mmdb", "GeoLite2-Country.mmdb"] {
    let path = mmdbs_path.join(file);

    if !path.exists() {
      let response = reqwest::get(format!("https://git.io/{file}")).await?;
      write(path, response.bytes().await?).await?;
      debug!("downloaded {file}")
    }
  }

  Ok(())
}
