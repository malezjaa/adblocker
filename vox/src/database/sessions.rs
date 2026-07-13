use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::RngExt;

use crate::database::DB;

pub fn generate_token() -> String {
  let bytes: [u8; 32] = rand::rng().random();
  hex::encode(bytes)
}

impl DB {
  pub async fn create_session(
    &self,
    token: String,
    ip: String,
    ttl_secs: i64,
  ) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
      "INSERT INTO session (token, created_at, expires_at, last_used, ip_address)
             VALUES (?, ?, ?, ?, ?)",
    )
    .bind(token)
    .bind(now)
    .bind(now + ttl_secs)
    .bind(now)
    .bind(ip)
    .execute(&self.pool)
    .await?;
    Ok(())
  }

  pub async fn validate_session(&self, token: String) -> Result<Option<DateTime<Utc>>> {
    let now = Utc::now().timestamp();

    let row: Option<(i64,)> = sqlx::query_as(
      r#"
        UPDATE session
        SET last_used = ?
        WHERE token = ? AND expires_at > ?
        RETURNING expires_at
        "#,
    )
    .bind(now)
    .bind(token)
    .bind(now)
    .fetch_optional(&self.pool)
    .await?;

    Ok(row.and_then(|(expires_at,)| DateTime::<Utc>::from_timestamp(expires_at, 0)))
  }

  pub async fn delete_session(&self, token: String) -> Result<()> {
    sqlx::query("DELETE FROM session WHERE token = ?")
      .bind(token)
      .execute(&self.pool)
      .await?;
    Ok(())
  }

  pub async fn cleanup_expired_sessions(&self) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query("DELETE FROM session WHERE expires_at <= ?")
      .bind(now)
      .execute(&self.pool)
      .await?;
    Ok(())
  }
}
