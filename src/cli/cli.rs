use clap::{Parser, Subcommand};
use dns_adblock::database::devices::DeviceType;

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(color = clap::ColorChoice::Always)]
pub struct Cli {
  #[arg(short, long)]
  pub verbose: bool,

  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
  Devices {
    #[command(subcommand)]
    command: DeviceCommand,
  },
}

#[derive(Subcommand, Debug)]
pub enum DeviceCommand {
  List,

  New {
    #[arg(long)]
    name: String,
    #[arg(long)]
    device_type: DeviceType,
  },

  Delete {
    #[arg()]
    id: String,
  },
}