pub mod crl;

use anyhow::{Context, Result, anyhow, bail};
use axum::response::IntoResponse;
use base64::Engine;
use base64::engine::general_purpose;
use fs_err::File;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};
use std::io::BufReader;
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub struct Certs {
  pub ca_cert: CertificateDer<'static>,
  pub certs: Vec<CertificateDer<'static>>,
  pub key: PrivateKeyDer<'static>,
}

impl Certs {
  pub fn load_certs() -> Result<Certs> {
    let certs_path = dirs::home_dir().unwrap().join("adb").join("certs");
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
      return Self::load(&ca_cert_path, &cert_path, &key_path);
    }

    fs_err::create_dir_all(&certs_path)?;
    Self::generate_certs(&certs_path)?;
    Self::install_in_cert_store(&ca_cert_path)?;
    Self::load(&ca_cert_path, &cert_path, &key_path)
  }

  fn create_open_ssl_config(certs_path: &Path) -> Result<()> {
    let cfg = certs_path.join("openssl.cnf");

    if !cfg.exists() {
      fs_err::write(cfg, include_str!("openssl.cnf"))?;
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
      .current_dir(&certs_path)
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
      .arg(&ca_cert_path)
      .status()
      .context("Failed to run certutil -addstore")?;

    if !trust_status.success() {
      bail!("certutil -addstore failed. try running as Administrator");
    }
    Ok(())
  }

  fn load(ca_cert_path: &Path, cert_path: &Path, key_path: &Path) -> Result<Certs> {
    let ca_cert =
      certs(&mut BufReader::new(File::open(ca_cert_path)?)).next().ok_or_else(
        || anyhow!("No CA certificate found in {}", ca_cert_path.display()),
      )??;
    let certs = certs(&mut BufReader::new(File::open(cert_path)?))
      .collect::<Result<Vec<_>, _>>()?;
    let key = private_key(&mut BufReader::new(File::open(key_path)?))?
      .ok_or_else(|| anyhow!("No private key found in {}", key_path.display()))?;
    Ok(Certs { ca_cert, certs, key })
  }
}
