use crate::database::DB;
use clap::ValueEnum;
use rand::{RngExt, distr::Alphanumeric};
use serde::{Deserialize, Serialize};

pub fn generate_device_id() -> String {
  rand::rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect()
}

#[derive(Debug, Clone, Copy, sqlx::Type, Serialize, Deserialize, ValueEnum)]
#[sqlx(type_name = "TEXT")]
pub enum DeviceType {
  #[sqlx(rename = "windows")]
  #[serde(rename = "windows")]
  Windows,
  #[sqlx(rename = "linux")]
  #[serde(rename = "linux")]
  Linux,
  #[sqlx(rename = "macos")]
  #[serde(rename = "macos")]
  MacOs,
  #[sqlx(rename = "ios")]
  #[serde(rename = "ios")]
  Ios,
  #[sqlx(rename = "android")]
  #[serde(rename = "android")]
  Android,
  #[sqlx(rename = "router")]
  #[serde(rename = "router")]
  Router,
  #[sqlx(rename = "other")]
  #[serde(rename = "other")]
  Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Device {
  pub id: String,
  pub name: String,
  #[sqlx(rename = "type")]
  pub device_type: DeviceType,
  pub last_seen: i64,
}

impl DB {
  pub async fn create_device(
    &self,
    name: &str,
    device_type: &str,
  ) -> Result<String, String> {
    const VALID_TYPES: &[&str] =
      &["windows", "linux", "macos", "ios", "android", "router", "other"];

    if name.trim().is_empty() {
      return Err("Device name cannot be empty".into());
    }

    if !VALID_TYPES.contains(&device_type) {
      return Err(format!("Invalid device type: {device_type}"));
    }

    self._create_device(name, DeviceType::from_str(device_type, true)?).await
  }

  pub async fn _create_device(
    &self,
    name: &str,
    device_type: DeviceType,
  ) -> Result<String, String> {
    let exists =
      sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM device WHERE name = ?)")
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to check device name: {e}"))?;

    if exists {
      return Err(format!("A device named '{name}' already exists"));
    }

    let id = generate_device_id();

    sqlx::query(
      "INSERT INTO device (id, name, type, last_seen)
               VALUES (?, ?, ?, strftime('%s', 'now'))",
    )
    .bind(&id)
    .bind(name)
    .bind(device_type)
    .execute(&self.pool)
    .await
    .map_err(|e| {
      if e.to_string().contains("UNIQUE") {
        "Generated device ID already exists, please try again".to_string()
      } else {
        format!("Failed to create device: {e}")
      }
    })?;

    Ok(id)
  }

  pub async fn get_devices(&self) -> Result<Vec<Device>, String> {
    sqlx::query_as::<_, Device>("SELECT id, name, type, last_seen FROM device")
      .fetch_all(&self.pool)
      .await
      .map_err(|e| format!("Failed to retrieve devices: {e}"))
  }

  pub async fn delete_device(&self, id: &str) -> Result<(), String> {
    let result = sqlx::query("DELETE FROM device WHERE id = ?")
      .bind(id)
      .execute(&self.pool)
      .await
      .map_err(|e| format!("Failed to delete device: {e}"))?;

    if result.rows_affected() == 0 {
      return Err(format!("No device found with id '{id}'"));
    }

    Ok(())
  }

  pub async fn get_device(&self, id: &str) -> Result<Device, String> {
    sqlx::query_as::<_, Device>(
      "SELECT id, name, type, last_seen FROM device WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| format!("Failed to retrieve device: {e}"))?
    .ok_or_else(|| format!("No device found with id '{id}'"))
  }
}
