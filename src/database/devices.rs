use crate::database::DB;
use rand::{Rng, distr::Alphanumeric};

pub fn generate_device_id() -> String {
  rand::rng().sample_iter(&Alphanumeric).take(8).map(char::from).collect()
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
}
