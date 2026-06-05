use anyhow::Result;
use fs_err::File;
use rcgen::{BasicConstraints, CertificateParams, CertifiedIssuer, DistinguishedName, DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};
use std::io::{BufReader, Cursor};
use time::{Duration, OffsetDateTime};

#[derive(Debug)]
pub struct Certs {
  pub certs: Vec<CertificateDer<'static>>,
  pub key: PrivateKeyDer<'static>,
}

impl Clone for Certs {
  fn clone(&self) -> Self {
    Self { certs: self.certs.clone(), key: self.key.clone_key() }
  }
}

pub fn get_certs() -> Result<Certs> {
  let certs_path = dirs::home_dir().unwrap().join("adb").join("certs");
  let ca_cert_path = certs_path.join("ca.pem");
  let cert_path = certs_path.join("cert.pem");
  let key_path = certs_path.join("key.pem");

  if cert_path.exists() && key_path.exists() {
    let certs = certs(&mut BufReader::new(File::open(&cert_path)?))
      .collect::<Result<Vec<_>, _>>()?;
    let key = private_key(&mut BufReader::new(File::open(&key_path)?))?
      .expect("No private key found");
    return Ok(Certs { certs, key });
  }

  fs_err::create_dir_all(&certs_path)?;

  // ca cert
  let ca_key = KeyPair::generate()?;
  let mut ca_params = CertificateParams::new(vec![])?;
  ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
  ca_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
  ca_params.not_before = OffsetDateTime::now_utc();
  ca_params.not_after = OffsetDateTime::now_utc() + Duration::days(3650);
  let mut dn = DistinguishedName::new();
  dn.push(DnType::CommonName, "DoT Local CA");
  ca_params.distinguished_name = dn;

  let ca = CertifiedIssuer::self_signed(ca_params, ca_key)?;
  fs_err::write(&ca_cert_path, ca.pem())?;

  // leaf cert signed by ca
  let leaf_key = KeyPair::generate()?;
  let mut leaf_params = CertificateParams::new(vec![
    "localhost".to_string(),
    "127.0.0.1".to_string(),
  ])?;
  leaf_params.is_ca = IsCa::NoCa;
  leaf_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
  leaf_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
  leaf_params.not_before = OffsetDateTime::now_utc();
  leaf_params.not_after = OffsetDateTime::now_utc() + Duration::days(365);
  let mut dn = DistinguishedName::new();
  dn.push(DnType::CommonName, "DoT Local");
  leaf_params.distinguished_name = dn;

  let leaf_cert = leaf_params.signed_by(&leaf_key, &ca)?;

  fs_err::write(&cert_path, leaf_cert.pem())?;
  fs_err::write(&key_path, leaf_key.serialize_pem())?;

  let certs = certs(&mut Cursor::new(leaf_cert.pem().as_bytes()))
    .collect::<Result<Vec<_>, _>>()?;
  let key = private_key(&mut Cursor::new(leaf_key.serialize_pem().as_bytes()))?
    .expect("No private key found");

  Ok(Certs { certs, key })
}