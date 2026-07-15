use std::{net::IpAddr, sync::atomic::Ordering, time::Duration};

use anyhow::Result;
use maxminddb::{Reader, path};
use tokio::time::sleep;
use tracing::error;

use crate::{context::Context, mmdb::downloader::DOWNLOADED_MMDBS};

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
    let ctx = self.clone();
    let paths = self.mmdb_paths();
    tokio::spawn(async move {
      loop {
        if DOWNLOADED_MMDBS.load(Ordering::Relaxed) {
          match (
            Reader::open_readfile(paths.asn.1.clone()),
            Reader::open_readfile(paths.country.1.clone()),
          ) {
            (Ok(asn), Ok(country)) => {
              ctx.replace_mmdbs(MMDBS { asn, country });
              break;
            }
            (Err(e), _) | (_, Err(e)) => {
              error!("Failed to load MMDBs: {e}");
            }
          }
        }
        sleep(Duration::from_secs(5)).await;
      }
    });

    Ok(())
  }

  pub fn lookup_mmdb(&self, ip: impl Into<String>) -> Result<Option<MMDBLookupResult>> {
    let mmdbs = self.mmdbs();
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
