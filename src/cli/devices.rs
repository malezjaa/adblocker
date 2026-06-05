use crate::CliContext;
use anyhow::{anyhow, Result};
use dns_adblock::database::devices::DeviceType;
use yansi::Paint;

pub fn pretty_device_type(ty: DeviceType) -> String {
  match ty {
    DeviceType::Windows => "Windows".rgb(56, 189, 248).to_string(),
    DeviceType::Linux => "Linux".rgb(251, 191, 36).to_string(),
    DeviceType::MacOs => "macOS".rgb(167, 139, 250).to_string(),
    DeviceType::Android => "Android".rgb(52, 211, 153).to_string(),
    DeviceType::Ios => "iOS".rgb(251, 113, 133).to_string(),
    DeviceType::Router => "Router".rgb(251, 146, 60).to_string(),
    DeviceType::Other => "Other".rgb(148, 163, 184).to_string(),
  }
}

impl CliContext {
  pub async fn new_device(&self, name: String, ty: DeviceType) -> Result<()> {
    let id = self.db._create_device(&name, ty).await
      .map_err(|err| anyhow!("couldn't create a new device: {err}"))?;

    let type_str = pretty_device_type(ty);

    println!();
    println!("  {} {}", "✓".green().bold(), "Device created".green().bold());
    println!("  {}", "─".repeat(30).dim());
    println!("  {}  {}", "Name:".dim(), name.bold());
    println!("  {}    {}", "ID:".dim(), id.cyan());
    println!("  {}  {}", "Type:".dim(), type_str);
    println!("  {}", "─".repeat(30).dim());

    Ok(())
  }
}