use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use tokio::net::TcpListener;
use vox_shared::home_dir;

#[derive(Clone)]
pub struct CrlPem(Vec<u8>);

async fn crl_pem_get(State(pem): State<CrlPem>) -> impl IntoResponse {
  ([("Content-Type", "application/pkix-crl")], Bytes::from(pem.0.clone()))
}

pub async fn serve_crl_pem() -> anyhow::Result<()> {
  let certs_path = home_dir().join("certs");
  let crl_path = certs_path.join("crl.pem");
  let bytes = fs_err::read(crl_path)?;

  let app = Router::new().route("/crl.pem", get(crl_pem_get)).with_state(CrlPem(bytes));

  let listener = TcpListener::bind("127.0.0.1:8080").await?;
  axum::serve(listener, app).await?;
  Ok(())
}
