use crate::CliContext;
use anyhow::Result;
use dns_adblock::context::Context;
use dns_adblock::firewall::override_dns::{OverrideDns, override_default_dns};
use tracing::warn;
use yansi::Paint;

pub async fn set_dns(
  ctx: &CliContext,
  device: Option<String>,
  no_doh: bool,
) -> Result<()> {
  let doh = if no_doh {
    warn!("it's not recommended to use unencrypted DNS");
    None
  } else {
    if let Some(device) = device {
      let device = ctx.db.get_device(&device).await?;
      Some(format!("https://127.0.0.1:443/dns-query/{}", device.id))
    } else {
      if ctx.config.dashboard_enabled() {
        warn!("setting no device means losing analytics")
      }
      Some("https://127.0.0.1:443/dns-query".to_owned())
    }
  };

  override_default_dns(OverrideDns {
    socket: Context::socket(),
    secondary: None,
    doh: doh.clone(),
  })?;

  println!();
  println!("  {} {}", "✓".green().bold(), "DNS configured".green().bold());
  println!("  {}", "─".repeat(44).dim());

  match &doh {
    Some(url) => {
      println!("  {}  {}", "Protocol:".dim(), "DNS-over-HTTPS".cyan().bold());
      println!("  {}  {}", "Endpoint:".dim(), url.bold());
    }
    None => {
      println!(
        "  {}  {}",
        "Protocol:".dim(),
        "Unencrypted DNS".rgb(251, 113, 133).bold()
      );
      println!("  {}  {}", "Endpoint:".dim(), "127.0.0.1 (system default)".bold());
    }
  }

  if no_doh {
    println!(
      "  {} {}",
      "⚠".rgb(251, 191, 36),
      "Unencrypted DNS is not recommended".rgb(251, 191, 36).dim()
    );
  }

  println!("  {}", "─".repeat(44).dim());
  println!();

  Ok(())
}
