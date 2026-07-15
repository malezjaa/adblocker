pub mod acme;
pub mod crl;
pub mod renewal;

use std::{io::Cursor, path::Path, process::Command};

use anyhow::{Context, Result, anyhow, bail};
use base64::{Engine, engine::general_purpose};
use fs_err::create_dir_all;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};
use vox_shared::{
  config::{Config, certs::CertificateStrategy},
  home_dir,
  path::canonicalize_with_strip,
};
use vox_windows::primary_adapter::primary_adapter;

use crate::dns::resolver::HickoryResolver;

#[derive(Debug)]
pub struct Certs {
  pub certs: Vec<CertificateDer<'static>>,
  pub key: PrivateKeyDer<'static>,
}

impl Certs {
  pub async fn load_certs(config: &Config, resolver: &HickoryResolver) -> Result<Certs> {
    match config.certs.strategy {
      CertificateStrategy::Manual => Self::load_manual(config),
      CertificateStrategy::Acme => Self::load_certs_with_acme(config, resolver).await,
      CertificateStrategy::SelfSigned => Self::load_self_signed(),
      CertificateStrategy::None => unreachable!(),
    }
  }

  fn load_self_signed() -> Result<Certs> {
    let openssl_available = Command::new("openssl")
      .arg("version")
      .output()
      .is_ok_and(|output| output.status.success());
    if !openssl_available {
      bail!("couldn't find openssl installed in the path.")
    }

    let certs_path = home_dir().join("certs").join("self_signed");
    create_dir_all(&certs_path)?;

    Self::create_open_ssl_config(&certs_path)?;

    let ca_cert_path = certs_path.join("ca.pem");
    let cert_path = certs_path.join("server.pem");
    let key_path = certs_path.join("server.key");
    let crl_path = certs_path.join("crl.pem");

    if ca_cert_path.exists()
      && cert_path.exists()
      && key_path.exists()
      && crl_path.exists()
    {
      return Self::load(&cert_path, &key_path);
    }

    create_dir_all(&certs_path)?;
    Self::generate_certs(&certs_path)?;
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

  fn create_open_ssl_config(certs_path: &Path) -> Result<()> {
    let cfg = certs_path.join("openssl.cnf");

    if !cfg.exists() {
      if let Some(adapter) = primary_adapter()? {
        let contents = include_str!("openssl.cnf")
          .replace("{HOST_IP}", &adapter.pick_ipv4()?.to_string());

        fs_err::create_dir_all(cfg.parent().unwrap())?;
        fs_err::write(cfg, contents)?;
      } else {
        bail!("Couldn't find a primary adapter")
      }
    }
    Ok(())
  }

  #[cfg(windows)]
  fn generate_certs(certs_path: &Path) -> Result<()> {
    let utf16: Vec<u8> = include_str!("gen-certs.ps1")
      .encode_utf16()
      .flat_map(|u| u.to_le_bytes())
      .collect();

    let encoded = general_purpose::STANDARD.encode(utf16);

    let status = Command::new("powershell")
      .arg("-ExecutionPolicy")
      .arg("Bypass")
      .arg("-EncodedCommand")
      .arg(encoded)
      .current_dir(certs_path)
      .status()?;
    if !status.success() {
      bail!("Certificate generation script exited with a non-zero status");
    }
    Ok(())
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
