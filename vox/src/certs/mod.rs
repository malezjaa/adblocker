pub mod acme;
pub mod openssl;
pub mod renewal;

use std::{io::Cursor, path::Path, process::Command};

use anyhow::{Context, Result, anyhow, bail};
use base64::{Engine, engine::general_purpose};
use fs_err::create_dir_all;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};
use tracing::info;
use vox_shared::{
  config::{Config, certs::CertificateStrategy},
  home_dir,
  path::canonicalize_with_strip,
};

use crate::certs::openssl::OpenSSL;

#[derive(Debug)]
pub struct Certs {
  pub certs: Vec<CertificateDer<'static>>,
  pub key: PrivateKeyDer<'static>,
}

impl Certs {
  pub fn load_certs(config: &Config) -> Result<Certs> {
    match config.certs.strategy {
      CertificateStrategy::Manual => Self::load_manual(config),
      CertificateStrategy::Acme => Self::load_certs_with_acme(config),
      CertificateStrategy::SelfSigned => Self::load_self_signed(),
      CertificateStrategy::None => unreachable!(),
    }
  }

  fn load_self_signed() -> Result<Certs> {
    let certs_path = home_dir().join("certs").join("self_signed");
    create_dir_all(&certs_path)?;

    let ca_key_path = certs_path.join("ca.key");
    let ca_cert_path = certs_path.join("ca.pem");
    let cert_path = certs_path.join("server.pem");
    let key_path = certs_path.join("server.key");

    if ca_cert_path.exists()
      && cert_path.exists()
      && key_path.exists()
      && ca_key_path.exists()
    {
      return Self::load(&cert_path, &key_path);
    }

    let openssl = OpenSSL::new(&ca_key_path, &ca_cert_path, &key_path, &cert_path)?;
    openssl.generate()?;
    info!("generated self-signed certs with openssl");

    Self::install_in_cert_store(&ca_cert_path)?;
    Self::load(&cert_path, &key_path)
  }

  fn load_manual(config: &Config) -> Result<Certs> {
    let (Some(cert_path), Some(key_path)) =
      (&config.certs.manual.cert_path, &config.certs.manual.key_path)
    else {
      bail!(
        "with certificate strategy set to manual you must provide both certificate and \
         key path"
      )
    };
    let cert_path = canonicalize_with_strip(cert_path)?;
    let key_path = canonicalize_with_strip(key_path)?;

    Self::load(&cert_path, &key_path)
  }

  #[cfg(windows)]
  fn install_in_cert_store(ca_cert_path: &Path) -> Result<()> {
    let trust_status = Command::new("certutil")
      .arg("-addstore")
      .arg("-f")
      .arg("Root")
      .arg(ca_cert_path)
      .status()
      .context("Failed to run certutil -addstore")?;

    if !trust_status.success() {
      bail!("certutil -addstore failed. try running as Administrator");
    }
    Ok(())
  }

  pub fn load(cert_path: &Path, key_path: &Path) -> Result<Certs> {
    Ok(Certs { certs: load_certs(cert_path)?, key: load_key(key_path)? })
  }
}

fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>> {
  let bytes = fs_err::read(path)?;

  Ok(certs(&mut Cursor::new(bytes)).collect::<Result<Vec<_>, _>>()?)
}

fn load_key(path: &Path) -> Result<PrivateKeyDer<'static>> {
  let bytes = fs_err::read(path)?;

  private_key(&mut Cursor::new(bytes))?
    .ok_or_else(|| anyhow!("No private key found in {}", path.display()))
}
