use crate::CliContext;
use crate::pretty::{print_error, print_warning};
use anyhow::Result;
use argon2::{
  Argon2, PasswordHasher, PasswordVerifier, password_hash::phc::PasswordHash,
};
use chrono::Utc;
use cliclack::{confirm, log, password};

pub fn hash_password(password: &str) -> String {
  Argon2::default().hash_password(password.as_bytes()).unwrap().to_string()
}

pub fn verify(password: &str, hash: &str) -> bool {
  let parsed = PasswordHash::new(hash).unwrap();
  Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok()
}

impl CliContext {
  pub async fn create_admin(&self) -> Result<()> {
    if self.db.admin_exists().await? {
      print_warning(
        "An admin account is already setup. If you wish to change password use change-password command",
      );
      return Ok(());
    }

    let pass = password("Enter admin password").mask('▪').interact()?;

    let pass2 = password("Confirm password").mask('▪').interact()?;

    if pass != pass2 {
      return Err(anyhow::anyhow!("Passwords do not match"));
    }

    let hash = hash_password(&pass);
    self.db.create_admin(&hash, Utc::now().timestamp()).await?;
    log::success("Created an admin account")?;
    Ok(())
  }

  pub async fn delete_admin(&self) -> Result<()> {
    if !self.db.admin_exists().await? {
      print_error("Admin account doesn't exist.");
      return Ok(());
    }

    if !confirm("Are you sure you want to delete admin account?").interact()? {
      return Ok(());
    }

    self.db.delete_admin().await?;
    log::success("Deleted admin account")?;
    Ok(())
  }

  pub async fn change_password(&self) -> Result<()> {
    if !self.db.admin_exists().await? {
      print_error("Admin account doesn't exist.");
      return Ok(());
    }

    let pass = password("Enter new admin password").mask('▪').interact()?;

    let pass2 = password("Confirm new password").mask('▪').interact()?;

    if pass != pass2 {
      return Err(anyhow::anyhow!("Passwords do not match"));
    }

    let hash = hash_password(&pass);
    self.db.create_admin(&hash, Utc::now().timestamp()).await?;
    log::success("Updated an admin account password")?;
    Ok(())
  }
}
