mod cli;
pub mod devices;

use crate::cli::{Cli, Commands, DeviceCommand, DnsCommand};
use anyhow::{Result, bail};
use clap::Parser;
use dns_adblock::config::Config;
use dns_adblock::context::Context;
use dns_adblock::database::DB;
use dns_adblock::firewall::override_dns::{OverrideDns, override_default_dns};
use dns_adblock::logger::setup_logger;
use tracing::{error, info, warn};

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
          doh,
        })?;

        info!("successfully overriden DNS settings");
        Ok(())
      }
    },
  };

  if let Err(err) = result {
    error!("{err}");
  }
  Ok(())
}
