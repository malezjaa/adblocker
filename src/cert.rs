use anyhow::Result;
use fs_err::File;
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};
use std::io::BufReader;
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
  let mut params =
    CertificateParams::new(vec!["localhost".to_string(), "127.0.0.1".to_string()])?;

  params.not_before = OffsetDateTime::now_utc();
  params.not_after = OffsetDateTime::now_utc() + Duration::days(365);

  let mut dn = DistinguishedName::new();
  dn.push(DnType::CommonName, "DoT Local");
  params.distinguished_name = dn;

  let key_pair = KeyPair::generate()?;
  let cert = params.self_signed(&key_pair)?;

  fs_err::create_dir_all("certs")?;
  fs_err::write("certs/cert.pem", cert.pem())?;
  fs_err::write("certs/key.pem", key_pair.serialize_pem())?;

  let certs = certs(&mut BufReader::new(File::open("certs/cert.pem")?))
    .collect::<Result<Vec<_>, _>>()?;
  let key = private_key(&mut BufReader::new(File::open("certs/key.pem")?))?
    .expect("No private key found");

  Ok(Certs { certs, key })
}
