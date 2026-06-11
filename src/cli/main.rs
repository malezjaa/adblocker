pub mod admin;
mod cli;
pub mod devices;
pub mod pretty;
pub mod set_dns;

use crate::cli::{AdminCommand, Cli, Commands, DeviceCommand, DnsCommand};
use crate::set_dns::set_dns;
use anyhow::{Result, bail};
use clap::Parser;
use cliclack::log;
use dns_adblock::config::Config;
use dns_adblock::database::DB;
use dns_adblock::logger::setup_logger;
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
      DnsCommand::Set { device, no_doh } => set_dns(&ctx, device, no_doh).await,
    },
    Commands::ResetDB => {
      ctx.db.reset_stats().await?;

      println!("  {} {}", "✓".green().bold(), "DB reset was successful".green().bold());
      Ok(())
    }
    Commands::Admin { command } => match command {
      AdminCommand::Create => ctx.create_admin().await,
      AdminCommand::Delete => ctx.delete_admin().await,
      AdminCommand::ChangePassword => ctx.change_password().await,
    },
  };

  if let Err(err) = result {
    log::error(format!("{err}"))?;
  }
  Ok(())
}
