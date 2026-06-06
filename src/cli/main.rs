mod cli;
pub mod devices;

use crate::cli::{Cli, Commands, DeviceCommand, DnsCommand};
use anyhow::{bail, Result};
use clap::Parser;
use dns_adblock::config::Config;
use dns_adblock::context::Context;
use dns_adblock::database::DB;
use dns_adblock::firewall::override_dns::{override_default_dns, OverrideDns};
use dns_adblock::logger::setup_logger;
use tracing::{error, info, warn};
use yansi::Paint;

#[derive(Debug)]
pub struct CliContext {
  pub db: DB,
  pub config: Config,
}

impl CliContext {
  pub async fn new() -> Result<Self> {
    let home_path = dirs::home_dir().unwrap().join("adb");

    let db_path = home_path.join("dns-adblock.sqlite");
    let db = DB::init(db_path).await?;

    let config_path = home_path.join("config.toml");
    let mut config = Config::from_file(&config_path)?;

    Ok(Self { db, config })
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  let cli = Cli::parse();
  setup_logger(cli.verbose);

  let ctx = CliContext::new().await?;

  let result = match cli.command {
    Commands::Devices { command } => match command {
      DeviceCommand::New { name, device_type } => ctx.new_device(name, device_type).await,
      DeviceCommand::List => ctx.list_devices().await,
      _ => bail!("not implemented"),
    },
    Commands::Dns { command } => match command {
      DnsCommand::Set { device, no_doh } => {
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
            println!("  {}  {}", "Protocol:".dim(), "Unencrypted DNS".rgb(251, 113, 133).bold());
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
    },
  };

  if let Err(err) = result {
    error!("{err}");
  }
  Ok(())
}
