use crate::pretty::{print_field, print_separator, print_success, print_warning};
use crate::CliContext;
use anyhow::Result;
use dns_adblock::context::Context;
use dns_adblock::firewall::override_dns::{override_default_dns, OverrideDns};
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

  print_success("DNS configured");
  print_separator(44);

  match &doh {
    Some(url) => {
      print_field("Protocol:", "DNS-over-HTTPS".cyan().bold());
      print_field("Endpoint:", url.bold());
    }
    None => {
      print_field("Protocol:", "Unencrypted DNS".rgb(251, 113, 133).bold());
      print_field("Endpoint:", "127.0.0.1 (system default)".bold());
    }
  }

  if no_doh {
    print_warning("Unencrypted DNS is not recommended");
  }

  print_separator(44);
  println!();

  Ok(())
}