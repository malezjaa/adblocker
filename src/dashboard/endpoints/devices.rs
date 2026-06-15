use crate::context::Context;
use crate::dashboard::AppError;
use crate::dashboard::auth::AuthGuard;
use crate::database::devices::Device;
use axum::Json;
use axum::extract::{Path, State as AxumState};
use serde::Deserialize;
use serde_json::{Value, json};

pub async fn get_devices_handler(
  _guard: AuthGuard,
  AxumState(ctx): AxumState<Context>,
) -> anyhow::Result<Json<Vec<Device>>, AppError> {
  let devices = ctx.db().get_devices().await?;
  Ok(Json(devices))
}

#[derive(Deserialize)]
pub struct CreateDevice {
  name: String,
  device_type: String,
}

pub async fn create_device_handler(
  _guard: AuthGuard,
  AxumState(ctx): AxumState<Context>,
  Json(body): Json<CreateDevice>,
) -> anyhow::Result<Json<Value>, AppError> {
  let id = ctx.db().create_device(&body.name, &body.device_type).await?;
  Ok(Json(json!({ "id": id })))
}

pub async fn get_device_handler(
  _guard: AuthGuard,
  AxumState(ctx): AxumState<Context>,
  Path(id): Path<String>,
) -> anyhow::Result<Json<Device>, AppError> {
  let device = ctx.db().get_device(&id).await?;
  Ok(Json(device))
}

pub async fn delete_device_handler(
  _guard: AuthGuard,
  AxumState(ctx): AxumState<Context>,
  Path(id): Path<String>,
) -> anyhow::Result<Json<Value>, AppError> {
  ctx.db().delete_device(&id).await?;
  Ok(Json(json!({ "success": true })))
}
