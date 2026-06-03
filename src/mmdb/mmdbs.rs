use crate::context::Context;
use crate::mmdb::downloader::DOWNLOADED_MMDBS;
use anyhow::Result;
use maxminddb::{path, Reader};
use std::net::IpAddr;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::time::sleep;
use tracing::error;

#[derive(Debug)]
pub struct MMDBS {
  pub asn: Reader<Vec<u8>>,
  pub country: Reader<Vec<u8>>,
}

#[derive(Debug)]
pub struct MMDBLookupResult {
  pub country: Option<String>,
  pub asn_org: Option<String>,
}

impl Context {
  pub fn load_mmdbs(&self) -> Result<()> {
    let inner = self.0.clone();
    tokio::spawn(async move {
      loop {
        if DOWNLOADED_MMDBS.load(Ordering::Relaxed) {
          match (
            Reader::open_readfile(inner.paths.asn.1.clone()),
            Reader::open_readfile(inner.paths.country.1.clone()),
          ) {
            (Ok(asn), Ok(country)) => {
              *inner.mmdbs.write() = Some(MMDBS { asn, country });
              break;
            }
            (Err(e), _) | (_, Err(e)) => {
              error!("Failed to load MMDBs: {e}");
            }
          }
        }
        sleep(Duration::from_secs(1)).await;
      }
    });

    Ok(())
  }

  pub fn lookup_mmdb(&self, ip: impl Into<String>) -> Result<Option<MMDBLookupResult>> {
    let mmdbs = self.0.mmdbs.read();
    let Some(mmdbs) = mmdbs.as_ref() else {
      return Ok(None);
    };

    let ip: IpAddr = ip.into().parse()?;

    let country_result = mmdbs.country.lookup(ip)?;
    let iso_code =
      country_result.decode_path(&path!["country", "iso_code"])?.map(str::to_string);

    let asn_result = mmdbs.asn.lookup(ip)?;
    let asn_org = asn_result
      .decode_path(&path!["autonomous_system_organization"])?
      .map(str::to_string);

    Ok(Some(MMDBLookupResult { country: iso_code, asn_org }))
  }
}
