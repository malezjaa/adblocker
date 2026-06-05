mod cli;
pub mod devices;

use crate::cli::{Cli, Commands, DeviceCommand};
use anyhow::{Result, bail};
use clap::Parser;
use dns_adblock::database::DB;
use dns_adblock::logger::setup_logger;
use tracing::error;

#[derive(Debug, Clone)]
pub struct CliContext {
  pub db: DB,
}

impl CliContext {
  pub async fn new() -> Result<Self> {
    let home_path = dirs::home_dir().unwrap().join("adb");

    let db_path = home_path.join("dns-adblock.sqlite");
    let db = DB::init(db_path).await?;

    Ok(Self { db })
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
  };

  if let Err(err) = result {
    error!("{err}");
  }
  Ok(())
}
