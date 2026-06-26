use anyhow::{Context, Result};
use axum::body::Bytes;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use fs_err::File;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};
use std::io::BufReader;
use std::path::Path;
use std::process::Command;
use tokio::net::TcpListener;

#[derive(Debug)]
pub struct Certs {
  pub ca_cert: CertificateDer<'static>,
  pub certs: Vec<CertificateDer<'static>>,
  pub key: PrivateKeyDer<'static>,
}

pub fn get_certs() -> Result<Certs> {
  let certs_path = dirs::home_dir().unwrap().join("adb").join("certs");
  let ca_cert_path = certs_path.join("ca.pem");
  let cert_path = certs_path.join("server.pem");
  let key_path = certs_path.join("server.key");
  let crl_path = certs_path.join("crl.pem");

  if ca_cert_path.exists() && cert_path.exists() && key_path.exists() && crl_path.exists() {
    return load_certs(&ca_cert_path, &cert_path, &key_path);
  }

  fs_err::create_dir_all(&certs_path)?;

  let gen_script = certs_path.join("gen-certs.ps1");
  let status = Command::new("powershell")
    .arg("-ExecutionPolicy").arg("Bypass")
    .arg("-File").arg(&gen_script)
    .current_dir(&certs_path)
    .status()
    .context("Failed to run cert generation script. is openssl on PATH?")?;
  if !status.success() {
    anyhow::bail!("Certificate generation script exited with a non-zero status");
  }

  let trust_status = Command::new("certutil")
    .arg("-addstore")
    .arg("-f")
    .arg("Root")
    .arg(&ca_cert_path)
    .status()
    .context("Failed to run certutil -addstore")?;
  if !trust_status.success() {
    anyhow::bail!("certutil -addstore failed. try running as Administrator");
  }

  load_certs(&ca_cert_path, &cert_path, &key_path)
}

fn load_certs(ca_cert_path: &Path, cert_path: &Path, key_path: &Path) -> Result<Certs> {
  let ca_cert = certs(&mut BufReader::new(File::open(ca_cert_path)?))
    .next()
    .ok_or_else(|| anyhow::anyhow!("No CA certificate found in {}", ca_cert_path.display()))??;
  let certs = certs(&mut BufReader::new(File::open(cert_path)?)).collect::<Result<Vec<_>, _>>()?;
  let key = private_key(&mut BufReader::new(File::open(key_path)?))?
    .ok_or_else(|| anyhow::anyhow!("No private key found in {}", key_path.display()))?;
  Ok(Certs { ca_cert, certs, key })
}

#[derive(Clone)]
pub struct CrlPem(Vec<u8>);

async fn crl_pem_get(State(pem): State<CrlPem>) -> impl IntoResponse {
  (
    [("Content-Type", "application/pkix-crl")],
    Bytes::from(pem.0.clone()),
  )
}

pub async fn serve_crl_pem() -> Result<()> {
  let certs_path = dirs::home_dir().unwrap().join("adb").join("certs");
  let crl_path = certs_path.join("crl.pem");
  let bytes = fs_err::read(crl_path)?;

  let app = Router::new()
    .route("/crl.pem", get(crl_pem_get))
    .with_state(CrlPem(bytes));

  let listener = TcpListener::bind("127.0.0.1:8080").await?;
  axum::serve(listener, app).await?;
  Ok(())
}