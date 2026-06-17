use crate::context::Context;
use crate::dashboard::AppError;
use crate::database::sessions::generate_token;
use crate::password::verify_password;
use anyhow::anyhow;
use axum::Json;
use axum::extract::{ConnectInfo, FromRef, FromRequestParts, State as AxumState};
use axum::http::header::SET_COOKIE;
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tracing::debug;

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

    let token = extract_session_cookie(&parts.headers)
      .ok_or_else(|| AppError::with_code("Unauthorized", StatusCode::UNAUTHORIZED))?;

    let valid = ctx.db().validate_session(token.to_owned()).await.unwrap_or(None);

    if let Some(time) = valid {
      debug!("valid session token until {}", time.format("%Y-%m-%d %H:%M:%S UTC"));
      Ok(AuthGuard)
    } else {
      Err(AppError::with_code("Unauthorized", StatusCode::UNAUTHORIZED))
    }
  }
}

pub fn extract_session_cookie(headers: &HeaderMap) -> Option<&str> {
  headers
    .get("Cookie")?
    .to_str()
    .ok()?
    .split(';')
    .map(|c| c.trim())
    .find_map(|c| c.strip_prefix("session="))
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

  let cookie =
    format!("session={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=86400", token);

  let response = Json(json!({ "success": true }));

  let mut res = response.into_response();
  res.headers_mut().insert(SET_COOKIE, HeaderValue::from_str(&cookie)?);

  Ok(res)
}

pub async fn auth_status(
  AxumState(ctx): AxumState<Context>,
  headers: HeaderMap,
) -> Result<Json<AuthStatus>, AppError> {
  let exists = ctx.db().admin_exists().await?;

  let logged_in = match extract_session_cookie(&headers) {
    Some(token) => ctx.db().validate_session(token.to_owned()).await.unwrap_or(None),
    None => None,
  };

  Ok(Json(AuthStatus { admin_exists: exists, logged_in: logged_in.is_some() }))
}

pub async fn auth_logout(
  AxumState(ctx): AxumState<Context>,
  headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
  let mut res = Json(json!({ "success": true })).into_response();

  if let Some(token) = extract_session_cookie(&headers) {
    ctx.db().delete_session(token.to_owned()).await?;
  }

  let cookie = "session=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax";

  res.headers_mut().insert(SET_COOKIE, HeaderValue::from_static(cookie));

  Ok(res)
}
