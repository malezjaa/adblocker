use anyhow::{Result, anyhow, bail};
use clap::ValueEnum;
use rand::{RngExt, distr::Alphanumeric};
use serde::{Deserialize, Serialize};

use crate::database::DB;

pub fn generate_device_id() -> String {
  rand::rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect()
}

#[derive(
  Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize, ValueEnum,
)]
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

#[derive(Debug, Clone, Serialize)]
pub struct DeviceRegistration {
  pub id: String,
  pub restored: bool,
}

#[derive(Debug, Clone)]
pub struct KnownDevice {
  pub id: String,
  pub name: String,
}

fn identifier_key(prefix: &str, value: &str) -> String {
  format!("{prefix}:{}", value.trim().to_ascii_lowercase())
}

impl DB {
  pub async fn create_device(
    &self,
    name: &str,
    device_type: &str,
  ) -> Result<DeviceRegistration> {
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
  ) -> Result<DeviceRegistration> {
    let name = name.trim();
    if name.is_empty() {
      bail!("Device name cannot be empty");
    }

    #[derive(sqlx::FromRow)]
    struct ExistingDevice {
      id: String,
      deleted_at: Option<i64>,
    }

    let existing = sqlx::query_as::<_, ExistingDevice>(
      "SELECT id, deleted_at
       FROM device
       WHERE name = ? COLLATE NOCASE
       ORDER BY deleted_at IS NULL DESC, deleted_at DESC
       LIMIT 1",
    )
    .bind(name)
    .fetch_optional(&self.pool)
    .await?;

    if let Some(existing) = existing {
      if existing.deleted_at.is_none() {
        bail!("A device named '{name}' already exists");
      }

      sqlx::query(
        "UPDATE device
         SET name = ?, type = ?, deleted_at = NULL
         WHERE id = ?",
      )
      .bind(name)
      .bind(device_type)
      .bind(&existing.id)
      .execute(&self.pool)
      .await?;

      self.cache_device_identity(&existing.id, name);

      return Ok(DeviceRegistration { id: existing.id, restored: true });
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

    self.cache_device_identity(&id, name);

    Ok(DeviceRegistration { id, restored: false })
  }

  pub async fn get_devices(&self) -> Result<Vec<Device>> {
    Ok(
      sqlx::query_as::<_, Device>(
        "SELECT id, name, type, last_seen
         FROM device
         WHERE deleted_at IS NULL
         ORDER BY name COLLATE NOCASE",
      )
      .fetch_all(&self.pool)
      .await?,
    )
  }

  pub async fn delete_device(&self, identifier: &str) -> Result<()> {
    let device = self.get_device_by_identifier(identifier).await?;
    let result = sqlx::query(
      "UPDATE device
       SET deleted_at = strftime('%s', 'now')
       WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(&device.id)
    .execute(&self.pool)
    .await?;

    if result.rows_affected() == 0 {
      bail!("No active device found for '{identifier}'");
    }

    self.remove_cached_device(&device);

    Ok(())
  }

  pub async fn get_device(&self, id: &str) -> Result<Device> {
    sqlx::query_as::<_, Device>(
      "SELECT id, name, type, last_seen
       FROM device
       WHERE id = ? COLLATE NOCASE AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?
    .ok_or_else(|| anyhow!("No device found with id '{id}'"))
  }

  pub async fn get_device_by_identifier(&self, identifier: &str) -> Result<Device> {
    sqlx::query_as::<_, Device>(
      "SELECT id, name, type, last_seen
       FROM device
       WHERE deleted_at IS NULL
         AND (id = ? COLLATE NOCASE OR name = ? COLLATE NOCASE)
       ORDER BY CASE WHEN id = ? COLLATE NOCASE THEN 0 ELSE 1 END
       LIMIT 1",
    )
    .bind(identifier)
    .bind(identifier)
    .bind(identifier)
    .fetch_optional(&self.pool)
    .await?
    .ok_or_else(|| anyhow!("No active device found for '{identifier}'"))
  }

  pub fn resolve_known_device(&self, identifier: &str) -> Option<KnownDevice> {
    self
      .known_devices
      .get(&identifier_key("id", identifier))
      .or_else(|| self.known_devices.get(&identifier_key("name", identifier)))
      .map(|entry| entry.value().clone())
  }

  pub(crate) fn cache_device(&self, device: &Device) {
    self.cache_device_identity(&device.id, &device.name);
  }

  fn cache_device_identity(&self, id: &str, name: &str) {
    let known = KnownDevice { id: id.to_owned(), name: name.to_owned() };
    self.known_devices.insert(identifier_key("id", id), known.clone());
    self.known_devices.insert(identifier_key("name", name), known);
  }

  fn remove_cached_device(&self, device: &Device) {
    self.known_devices.remove(&identifier_key("id", &device.id));
    self.known_devices.remove(&identifier_key("name", &device.name));
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::*;

  async fn test_db() -> (DB, PathBuf) {
    let path = std::env::temp_dir()
      .join(format!("vox-device-test-{}.sqlite", generate_device_id()));
    let db = DB::init_simple(&path).await.unwrap();
    (db, path)
  }

  async fn cleanup(db: DB, path: PathBuf) {
    db.pool.close().await;
    let _ = fs_err::remove_file(path);
  }

  #[tokio::test]
  async fn devices_can_be_selected_by_name_or_id() {
    let (db, path) = test_db().await;
    let registration =
      db._create_device("  Office Laptop  ", DeviceType::Linux).await.unwrap();

    let by_name = db.get_device_by_identifier("office laptop").await.unwrap();
    let by_id =
      db.get_device_by_identifier(&registration.id.to_ascii_lowercase()).await.unwrap();

    assert_eq!(by_name.id, registration.id);
    assert_eq!(by_id.id, registration.id);
    assert_eq!(by_name.name, "Office Laptop");
    assert_eq!(db.resolve_known_device("OFFICE LAPTOP").unwrap().id, registration.id);

    cleanup(db, path).await;
  }

  #[tokio::test]
  async fn adding_an_archived_name_restores_its_id_and_history() {
    let (db, path) = test_db().await;
    let original = db._create_device("Media Box", DeviceType::Linux).await.unwrap();

    sqlx::query(
      "INSERT INTO query_log (
          domain, record_type, client_ip, blocked, response_code,
          timestamp, response_time, device_id
       ) VALUES ('example.test', 'A', '127.0.0.1', 0, 'NoError', 1, 1, ?)",
    )
    .bind(&original.id)
    .execute(&db.pool)
    .await
    .unwrap();

    db.delete_device("media box").await.unwrap();
    assert!(db.get_devices().await.unwrap().is_empty());
    assert!(db.resolve_known_device(&original.id).is_none());

    let restored = db._create_device("MEDIA BOX", DeviceType::Other).await.unwrap();
    let device = db.get_device(&restored.id).await.unwrap();
    let logged_device_id: Option<String> =
      sqlx::query_scalar("SELECT device_id FROM query_log LIMIT 1")
        .fetch_one(&db.pool)
        .await
        .unwrap();

    assert!(restored.restored);
    assert_eq!(restored.id, original.id);
    assert_eq!(device.device_type, DeviceType::Other);
    assert_eq!(logged_device_id.as_deref(), Some(original.id.as_str()));
    assert_eq!(db.resolve_known_device(&original.id).unwrap().name, "MEDIA BOX");

    cleanup(db, path).await;
  }

  #[tokio::test]
  async fn active_device_names_are_unique_ignoring_case() {
    let (db, path) = test_db().await;
    db._create_device("Router", DeviceType::Router).await.unwrap();

    let err = db._create_device("router", DeviceType::Other).await.unwrap_err();

    assert!(err.to_string().contains("already exists"));

    cleanup(db, path).await;
  }

  #[tokio::test]
  async fn device_cache_updates_are_visible_to_db_clones() {
    let (db, path) = test_db().await;
    let dns_worker_db = db.clone();

    let registration = db._create_device("PC", DeviceType::Windows).await.unwrap();

    assert_eq!(dns_worker_db.resolve_known_device("PC").unwrap().id, registration.id);
    assert_eq!(dns_worker_db.resolve_known_device(&registration.id).unwrap().name, "PC");

    db.delete_device("PC").await.unwrap();

    assert!(dns_worker_db.resolve_known_device("PC").is_none());
    assert!(dns_worker_db.resolve_known_device(&registration.id).is_none());

    cleanup(db, path).await;
  }

  #[tokio::test]
  async fn existing_device_tables_gain_the_archive_column() {
    let path = std::env::temp_dir()
      .join(format!("vox-device-migration-test-{}.sqlite", generate_device_id()));
    let pool = DB::init_db(&path).await.unwrap();

    sqlx::query(
      "CREATE TABLE device (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL,
          type TEXT NOT NULL,
          last_seen INTEGER
       )",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool.close().await;

    let db = DB::init_simple(&path).await.unwrap();
    let has_deleted_at: bool = sqlx::query_scalar(
      "SELECT EXISTS(
          SELECT 1
          FROM pragma_table_info('device')
          WHERE name = 'deleted_at'
       )",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();

    assert!(has_deleted_at);

    cleanup(db, path).await;
  }
}
