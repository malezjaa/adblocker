use crate::context::Context;
use crate::mmdb::downloader::DOWNLOADED_MMDBS;
use anyhow::Result;
use maxminddb::{Reader, path};
use std::net::IpAddr;
use std::sync::atomic::Ordering;

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
    tokio::spawn(async move {
      loop {
        if DOWNLOADED_MMDBS.load(Ordering::Relaxed) {
          println!("downloaded",);
          break;
        }
      }
    });
    let asn = Reader::open_readfile(self.0.paths.asn.1.clone())?;
    let country = Reader::open_readfile(self.0.paths.country.1.clone())?;

    *self.0.mmdbs.write() = Some(MMDBS { asn, country });
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
