use std::path::Path;

use sqlx::{
  SqlitePool,
  sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use crate::database::DB;

impl DB {
  pub async fn init_db(path: &Path) -> anyhow::Result<SqlitePool> {
    let options = SqliteConnectOptions::new()
      .filename(path)
      .create_if_missing(true)
      .foreign_keys(true);

    Ok(SqlitePoolOptions::new().max_connections(5).connect_with(options).await?)
  }

  pub async fn init_schema(&self) -> anyhow::Result<()> {
    sqlx::query(
      r#"
      CREATE TABLE IF NOT EXISTS query_log (
          id            INTEGER PRIMARY KEY AUTOINCREMENT,
          domain        TEXT    NOT NULL,
          record_type   TEXT    NOT NULL,
          client_ip     TEXT    NOT NULL,
          blocked       INTEGER NOT NULL,
          block_origin  INTEGER,
          response_code TEXT    NOT NULL,
          timestamp     INTEGER NOT NULL,
          response_time INTEGER NOT NULL,
          country_code  TEXT,
          company_name  TEXT,
          device_id     TEXT,

          FOREIGN KEY (device_id)
              REFERENCES device(id)
              ON DELETE SET NULL
              ON UPDATE CASCADE
      );
      "#,
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      r#"
      CREATE INDEX IF NOT EXISTS idx_query_log_blocked_timestamp
          ON query_log(blocked, timestamp);
      "#,
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      r#"
      CREATE INDEX IF NOT EXISTS idx_query_log_domain
          ON query_log(domain);
      "#,
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      r#"
      CREATE INDEX IF NOT EXISTS idx_query_log_record_type
          ON query_log(record_type);
      "#,
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      r#"
      CREATE TABLE IF NOT EXISTS domain_stats (
          domain            TEXT    PRIMARY KEY,
          registered_domain TEXT    NOT NULL,
          hits_total        INTEGER NOT NULL,
          hits_blocked      INTEGER NOT NULL,
          last_seen         INTEGER NOT NULL
      );
      "#,
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      r#"
      CREATE INDEX IF NOT EXISTS idx_domain_stats_registered
          ON domain_stats(registered_domain);
      "#,
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      r#"
      CREATE TABLE IF NOT EXISTS device (
          id         TEXT PRIMARY KEY,
          name       TEXT NOT NULL,
          type       TEXT NOT NULL CHECK (
              type IN (
                  'windows',
                  'linux',
                  'macos',
                  'ios',
                  'android',
                  'router',
                  'other'
              )
          ),
          last_seen  INTEGER,
          deleted_at INTEGER
      );
      "#,
    )
    .execute(&self.pool)
    .await?;

    let has_deleted_at: bool = sqlx::query_scalar(
      "SELECT EXISTS(
          SELECT 1
          FROM pragma_table_info('device')
          WHERE name = 'deleted_at'
      )",
    )
    .fetch_one(&self.pool)
    .await?;

    if !has_deleted_at {
      sqlx::query("ALTER TABLE device ADD COLUMN deleted_at INTEGER")
        .execute(&self.pool)
        .await?;
    }

    sqlx::query(
      r#"
      CREATE INDEX IF NOT EXISTS idx_device_active_name
          ON device(name COLLATE NOCASE)
          WHERE deleted_at IS NULL;
      "#,
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      r#"
      CREATE INDEX IF NOT EXISTS idx_domain_stats_last_seen
          ON domain_stats(last_seen);
      "#,
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      r#"
      CREATE TABLE IF NOT EXISTS admin (
          id            INTEGER PRIMARY KEY CHECK (id = 1),
          password_hash TEXT    NOT NULL,
          created_at    INTEGER NOT NULL,
          last_login    INTEGER
      );
      "#,
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      r#"
      CREATE TABLE IF NOT EXISTS session (
          token      TEXT    PRIMARY KEY,
          created_at INTEGER NOT NULL,
          expires_at INTEGER NOT NULL,
          last_used  INTEGER NOT NULL,
          ip_address TEXT    NOT NULL
      );
      "#,
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      r#"
      CREATE INDEX IF NOT EXISTS idx_session_expires_at
          ON session(expires_at);
      "#,
    )
    .execute(&self.pool)
    .await?;

    Ok(())
  }
}
