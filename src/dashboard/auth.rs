use crate::context::Context;
use crate::dashboard::AppError;
use crate::database::sessions::generate_token;
use crate::password::verify_password;
use anyhow::anyhow;
use axum::Json;
use axum::extract::{ConnectInfo, FromRef, FromRequestParts, State as AxumState};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::net::SocketAddr;

pub struct AuthGuard;

impl<S> FromRequestParts<S> for AuthGuard
where
  S: Send + Sync,
  Context: FromRef<S>,
{
  type Rejection = AppError;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &S,
  ) -> Result<Self, Self::Rejection> {
    let ctx = Context::from_ref(state);

    let token = extract_bearer(&parts.headers)
      .ok_or_else(|| AppError::with_code("Unauthorized", StatusCode::UNAUTHORIZED))?;

    let valid = ctx.db().validate_session(token.to_owned()).await.unwrap_or(false);

    if !valid {
      return Err(AppError::with_code("Unauthorized", StatusCode::UNAUTHORIZED));
    }

    Ok(AuthGuard)
  }
}

pub fn extract_bearer(headers: &HeaderMap) -> Option<&str> {
  headers
    .get("Authorization")
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.strip_prefix("Bearer "))
}

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
  password: String,
}

#[derive(Serialize)]
pub struct AuthStatus {
  admin_exists: bool,
  logged_in: bool,
}

pub async fn auth_login(
  AxumState(ctx): AxumState<Context>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
  let row: Option<(String,)> =
    sqlx::query_as("SELECT password_hash FROM admin WHERE id = 1")
      .fetch_optional(&ctx.db().pool)
      .await?;

  let Some((stored_hash,)) = row else {
    return Err(anyhow!("Admin not initialized").into());
  };

  if !verify_password(&body.password, &stored_hash) {
    return Err(anyhow!("Invalid credentials").into());
  }

  let token = generate_token();
  ctx.db().create_session(token.clone(), addr.ip().to_string(), 86400).await?;

  Ok(Json(json!({ "success": true, "token": token })))
}

pub async fn auth_status(
  AxumState(ctx): AxumState<Context>,
  headers: HeaderMap,
) -> Result<Json<AuthStatus>, AppError> {
  let exists = ctx.db().admin_exists().await?;

  let logged_in = match extract_bearer(&headers) {
    Some(token) => ctx.db().validate_session(token.to_owned()).await.unwrap_or(false),
    None => false,
  };

  Ok(Json(AuthStatus { admin_exists: exists, logged_in }))
}

pub async fn auth_logout(
  AxumState(ctx): AxumState<Context>,
  headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
  if let Some(token) = extract_bearer(&headers) {
    ctx.db().delete_session(token.to_owned()).await?;
  }
  Ok(Json(json!({ "success": true })))
}
