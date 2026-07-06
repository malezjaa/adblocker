use anyhow::{anyhow, Result};
use rustls::pki_types::CertificateDer;
use ::time::{Duration, OffsetDateTime};
use x509_parser::prelude::*;

pub fn certs_need_renewal(certs: &[CertificateDer<'static>]) -> Result<bool> {
  let leaf = certs
    .first()
    .ok_or_else(|| anyhow!("certificate chain is empty"))?;

  let (_, cert) = X509Certificate::from_der(leaf.as_ref())
    .map_err(|err| anyhow!("failed to parse certificate: {err}"))?;

  let not_after = cert
    .validity()
    .not_after
    .to_datetime();

  let renew_at = OffsetDateTime::now_utc() + Duration::days(30);

  Ok(not_after <= renew_at)
}