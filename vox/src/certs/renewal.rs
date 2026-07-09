use ::time::{Duration, OffsetDateTime};
use anyhow::{Result, anyhow};
use rustls::pki_types::CertificateDer;
use x509_parser::prelude::*;

pub fn certs_need_renewal(certs: &[CertificateDer<'static>]) -> Result<bool> {
  let leaf = certs.first().ok_or_else(|| anyhow!("certificate chain is empty"))?;

  let (_, cert) = X509Certificate::from_der(leaf.as_ref())
    .map_err(|err| anyhow!("failed to parse certificate: {err}"))?;

  let not_after = cert.validity().not_after.to_datetime();

  let renew_at = OffsetDateTime::now_utc() + Duration::days(30);

  Ok(not_after <= renew_at)
}

#[cfg(test)]
mod tests {
  use super::*;
  use ::time::{Duration, OffsetDateTime};
  use rcgen::{CertificateParams, KeyPair};

  fn cert_expiring_in(days: i64) -> CertificateDer<'static> {
    let now = OffsetDateTime::now_utc();
    let mut params = CertificateParams::new(vec!["localhost".into()]).unwrap();
    params.not_before = now - Duration::days(1);
    params.not_after = now + Duration::days(days);
    let key = KeyPair::generate().unwrap();
    let cert = params.self_signed(&key).unwrap();

    CertificateDer::from(cert.der().to_vec())
  }

  #[test]
  fn empty_certificate_chain_returns_error() {
    let err = certs_need_renewal(&[]).unwrap_err();

    assert!(err.to_string().contains("certificate chain is empty"));
  }

  #[test]
  fn certificate_expiring_within_thirty_days_needs_renewal() {
    let cert = cert_expiring_in(10);

    assert!(certs_need_renewal(&[cert]).unwrap());
  }

  #[test]
  fn certificate_expiring_after_thirty_days_does_not_need_renewal() {
    let cert = cert_expiring_in(60);

    assert!(!certs_need_renewal(&[cert]).unwrap());
  }

  #[test]
  fn invalid_certificate_der_returns_parse_error() {
    let cert = CertificateDer::from(vec![0, 1, 2, 3]);
    let err = certs_need_renewal(&[cert]).unwrap_err();

    assert!(err.to_string().contains("failed to parse certificate"));
  }
}
