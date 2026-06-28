use crate::database::DB;

impl DB {
  pub async fn admin_exists(&self) -> Result<bool, sqlx::Error> {
    let exists: (i64,) = sqlx::query_as("SELECT COUNT(1) FROM admin WHERE id = 1")
      .fetch_one(&self.pool)
      .await?;

    Ok(exists.0 > 0)
  }

  pub async fn create_admin(
    &self,
    password_hash: &str,
    created_at: i64,
  ) -> Result<(), sqlx::Error> {
    sqlx::query(
      r#"
        INSERT INTO admin (id, password_hash, created_at, last_login)
        VALUES (1, ?1, ?2, NULL)
        ON CONFLICT(id) DO UPDATE SET
            password_hash = excluded.password_hash,
            created_at = excluded.created_at
        "#,
    )
    .bind(password_hash)
    .bind(created_at)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  pub async fn delete_admin(&self) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM admin WHERE id = 1").execute(&self.pool).await?;

    Ok(())
  }
}
