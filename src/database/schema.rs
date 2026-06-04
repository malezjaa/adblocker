use crate::database::DB;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::Path;

impl DB {
  pub async fn init_db(path: &Path) -> anyhow::Result<SqlitePool> {
    let options = SqliteConnectOptions::new().filename(path).create_if_missing(true);

    Ok(SqlitePoolOptions::new().max_connections(5).connect_with(options).await?)
  }

  pub async fn init_schema(&self) -> anyhow::Result<()> {
    sqlx::query(
      "CREATE TABLE IF NOT EXISTS query_log (
         id            INTEGER PRIMARY KEY AUTOINCREMENT,
         domain        TEXT    NOT NULL,
         client_ip     TEXT    NOT NULL,
         blocked       INTEGER NOT NULL,
         block_origin  TEXT,
         timestamp     INTEGER NOT NULL,
         response_time INTEGER NOT NULL
       )",
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      "CREATE INDEX IF NOT EXISTS idx_query_log_blocked_timestamp
             ON query_log(blocked, timestamp)",
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      "CREATE INDEX IF NOT EXISTS idx_query_log_domain
             ON query_log(domain)",
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      "CREATE TABLE IF NOT EXISTS domain_stats (
              domain             TEXT    PRIMARY KEY,
              registered_domain  TEXT    NOT NULL,
              hits_total         INTEGER NOT NULL,
              hits_blocked       INTEGER NOT NULL,
              last_seen          INTEGER NOT NULL
            );",
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      "CREATE INDEX IF NOT EXISTS idx_domain_stats_registered
                ON domain_stats(registered_domain);",
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      "CREATE TABLE IF NOT EXISTS device (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      type TEXT NOT NULL CHECK (
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
      last_seen INTEGER
    );",
    )
    .execute(&self.pool)
    .await?;

    sqlx::query(
      "CREATE INDEX IF NOT EXISTS idx_domain_stats_last_seen
             ON domain_stats(last_seen)",
    )
    .execute(&self.pool)
    .await?;

    Ok(())
  }
}
