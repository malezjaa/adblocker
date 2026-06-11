use crate::database::DB;
use rand::{Rng, RngExt};

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
  ) -> anyhow::Result<()> {
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

  pub async fn validate_session(&self, token: String) -> anyhow::Result<bool> {
    let now = chrono::Utc::now().timestamp();
    let row: Option<(i64,)> = sqlx::query_as(
      "UPDATE session SET last_used = ?
             WHERE token = ? AND expires_at > ?
             RETURNING expires_at",
    )
    .bind(now)
    .bind(token)
    .bind(now)
    .fetch_optional(&self.pool)
    .await?;
    Ok(row.is_some())
  }

  pub async fn delete_session(&self, token: String) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM session WHERE token = ?")
      .bind(token)
      .execute(&self.pool)
      .await?;
    Ok(())
  }

  pub async fn cleanup_expired_sessions(&self) -> anyhow::Result<()> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query("DELETE FROM session WHERE expires_at <= ?")
      .bind(now)
      .execute(&self.pool)
      .await?;
    Ok(())
  }
}
