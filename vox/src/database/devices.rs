use crate::database::DB;
use anyhow::Result;
use anyhow::{anyhow, bail};
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
  pub async fn create_device(&self, name: &str, device_type: &str) -> Result<String> {
    if name.trim().is_empty() {
      bail!("Device name cannot be empty");
    }

    let device_type = DeviceType::from_str(device_type, true)
      .map_err(|_| anyhow!("Invalid device type: {device_type}"))?;

    self._create_device(name, device_type).await
  }

  pub async fn _create_device(
    &self,
    name: &str,
    device_type: DeviceType,
  ) -> Result<String> {
    let exists =
      sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM device WHERE name = ?)")
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

    if exists {
      bail!("A device named '{name}' already exists");
    }

    let id = generate_device_id();

    match sqlx::query(
      "INSERT INTO device (id, name, type, last_seen)
       VALUES (?, ?, ?, strftime('%s', 'now'))",
    )
    .bind(&id)
    .bind(name)
    .bind(device_type)
    .execute(&self.pool)
    .await
    {
      Ok(_) => {}
      Err(e) if e.to_string().contains("UNIQUE") => {
        bail!("Generated device ID already exists, please try again");
      }
      Err(e) => return Err(e.into()),
    }

    self.known_devices.insert(id.clone());

    Ok(id)
  }

  pub async fn get_devices(&self) -> Result<Vec<Device>> {
    Ok(
      sqlx::query_as::<_, Device>("SELECT id, name, type, last_seen FROM device")
        .fetch_all(&self.pool)
        .await?,
    )
  }

  pub async fn delete_device(&self, id: &str) -> Result<()> {
    let result =
      sqlx::query("DELETE FROM device WHERE id = ?").bind(id).execute(&self.pool).await?;

    if result.rows_affected() == 0 {
      bail!("No device found with id '{id}'");
    }

    self.known_devices.remove(id);

    Ok(())
  }

  pub async fn get_device(&self, id: &str) -> Result<Device> {
    sqlx::query_as::<_, Device>(
      "SELECT id, name, type, last_seen FROM device WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?
    .ok_or_else(|| anyhow!("No device found with id '{id}'"))
  }
}
