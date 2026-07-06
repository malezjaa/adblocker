use crate::CliContext;
use anyhow::{bail, Result};
use chrono::Utc;
use cliclack::{confirm, log, password};
use vox::password::hash_password;
use vox_shared::pretty::{print_error, print_warning};

impl CliContext {
  pub async fn create_admin(&self) -> Result<()> {
    if self.db.admin_exists().await? {
      print_warning(
        "An admin account is already setup. If you wish to change password use change-password command",
      );
      return Ok(());
    }

    let pass = password("Enter admin password").mask('▪').interact()?;

    if pass.len() < 8 {
      bail!("Password must be at least 8 characters long");
    }
    if !pass.chars().any(|c| c.is_uppercase()) {
      bail!("Password must contain at least one uppercase letter");
    }
    if !pass.chars().any(|c| c.is_numeric()) {
      bail!("Password must contain at least one number");
    }

    let pass2 = password("Confirm password").mask('▪').interact()?;

    if pass != pass2 {
      bail!("Passwords do not match");
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
