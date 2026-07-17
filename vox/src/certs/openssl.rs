use std::{net::Ipv4Addr, path::Path};

use anyhow::{Result, bail};
use openssl::{
  asn1::Asn1Time,
  bn::{BigNum, MsbOption},
  hash::MessageDigest,
  pkey::{PKey, Private},
  rsa::Rsa,
  x509::{
    X509, X509NameBuilder, X509Req, X509ReqBuilder, X509VerifyResult,
    extension::{
      AuthorityKeyIdentifier, BasicConstraints, ExtendedKeyUsage, KeyUsage,
      SubjectAlternativeName, SubjectKeyIdentifier,
    },
  },
};
use vox_windows::primary_adapter::primary_adapter;

pub struct OpenSSL<'a> {
  pub ca_key_path: &'a Path,
  pub ca_cert_path: &'a Path,

  pub server_key_path: &'a Path,
  pub server_cert_path: &'a Path,

  pub host_ip: Ipv4Addr,
}

impl<'a> OpenSSL<'a> {
  pub fn new(
    ca_key_path: &'a Path,
    ca_cert_path: &'a Path,
    server_key_path: &'a Path,
    server_cert_path: &'a Path,
  ) -> Result<Self> {
    if let Some(adapter) = primary_adapter()? {
      Ok(Self {
        host_ip: adapter.pick_ipv4()?,
        ca_cert_path,
        ca_key_path,
        server_cert_path,
        server_key_path,
      })
    } else {
      bail!("Couldn't find a primary adapter")
    }
  }

  fn build_ca(&self, ca_key: &PKey<Private>) -> Result<X509> {
    let mut x509_name = X509NameBuilder::new()?;
    x509_name.append_entry_by_text("O", "Vox CA")?;
    x509_name.append_entry_by_text("CN", "Vox CA")?;
    let x509_name = x509_name.build();

    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    let serial_number = {
      let mut serial = BigNum::new()?;
      serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
      serial.to_asn1_integer()?
    };
    cert_builder.set_serial_number(&serial_number)?;
    cert_builder.set_subject_name(&x509_name)?;
    cert_builder.set_issuer_name(&x509_name)?;
    cert_builder.set_pubkey(&ca_key)?;
    let not_before = Asn1Time::days_from_now(0)?;
    cert_builder.set_not_before(&not_before)?;
    let not_after = Asn1Time::days_from_now(3650)?;
    cert_builder.set_not_after(&not_after)?;

    cert_builder.append_extension(BasicConstraints::new().critical().ca().build()?)?;
    cert_builder
      .append_extension(KeyUsage::new().critical().key_cert_sign().crl_sign().build()?)?;

    let subject_key_identifier =
      SubjectKeyIdentifier::new().build(&cert_builder.x509v3_context(None, None))?;
    cert_builder.append_extension(subject_key_identifier)?;

    cert_builder.sign(&ca_key, MessageDigest::sha256())?;
    let cert = cert_builder.build();

    Ok(cert)
  }

  fn mk_request(&self, key_pair: &PKey<Private>) -> Result<X509Req> {
    let mut req_builder = X509ReqBuilder::new()?;
    req_builder.set_pubkey(key_pair)?;

    let mut x509_name = X509NameBuilder::new()?;
    x509_name.append_entry_by_text("O", "Vox Server")?;
    x509_name.append_entry_by_text("CN", "doh.local")?;
    let x509_name = x509_name.build();
    req_builder.set_subject_name(&x509_name)?;

    req_builder.sign(key_pair, MessageDigest::sha256())?;
    let req = req_builder.build();
    Ok(req)
  }

  fn make_ca_signed_cert(
    &self,
    ca_key: &PKey<Private>,
    ca_cert: &X509,
  ) -> Result<(X509, PKey<Private>)> {
    let rsa = Rsa::generate(2048)?;
    let key_pair = PKey::from_rsa(rsa)?;

    let req = self.mk_request(&key_pair)?;

    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    let serial_number = {
      let mut serial = BigNum::new()?;
      serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
      serial.to_asn1_integer()?
    };
    cert_builder.set_serial_number(&serial_number)?;
    cert_builder.set_subject_name(req.subject_name())?;
    cert_builder.set_issuer_name(ca_cert.subject_name())?;
    cert_builder.set_pubkey(&key_pair)?;
    let not_before = Asn1Time::days_from_now(0)?;
    cert_builder.set_not_before(&not_before)?;
    let not_after = Asn1Time::days_from_now(365)?;
    cert_builder.set_not_after(&not_after)?;

    cert_builder.append_extension(BasicConstraints::new().critical().build()?)?;
    cert_builder.append_extension(ExtendedKeyUsage::new().server_auth().build()?)?;

    cert_builder.append_extension(
      KeyUsage::new()
        .critical()
        .non_repudiation()
        .digital_signature()
        .key_encipherment()
        .build()?,
    )?;

    let subject_key_identifier = SubjectKeyIdentifier::new()
      .build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
    cert_builder.append_extension(subject_key_identifier)?;

    let auth_key_identifier = AuthorityKeyIdentifier::new()
      .keyid(false)
      .issuer(false)
      .build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
    cert_builder.append_extension(auth_key_identifier)?;

    let subject_alt_name = SubjectAlternativeName::new()
      .dns("doh.local")
      .dns("localhost")
      .ip("127.0.0.1")
      .ip("::1")
      .ip(&self.host_ip.to_string())
      .build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
    cert_builder.append_extension(subject_alt_name)?;

    cert_builder.sign(ca_key, MessageDigest::sha256())?;
    let cert = cert_builder.build();

    Ok((cert, key_pair))
  }

  pub fn generate(&self) -> Result<()> {
    let ca_key = PKey::from_rsa(Rsa::generate(4096)?)?;
    let ca_cert = self.build_ca(&ca_key)?;

    let (server_cert, server_key) = self.make_ca_signed_cert(&ca_key, &ca_cert)?;
    assert_eq!(ca_cert.issued(&server_cert), X509VerifyResult::OK);

    fs_err::write(self.ca_key_path, ca_key.private_key_to_pem_pkcs8()?)?;
    fs_err::write(self.ca_cert_path, ca_cert.to_pem()?)?;
    fs_err::write(self.server_key_path, server_key.private_key_to_pem_pkcs8()?)?;
    fs_err::write(self.server_cert_path, server_cert.to_pem()?)?;
    Ok(())
  }
}
