use clap::{Parser, Subcommand};
use vox::database::devices::DeviceType;

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
  Dns {
    #[command(subcommand)]
    command: DnsCommand,
  },
  #[command(name = "reset-db")]
  ResetDB,

  Admin {
    #[command(subcommand)]
    command: AdminCommand,
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
    /// Device name or ID.
    #[arg()]
    device: String,
  },
}

#[derive(Subcommand, Debug)]
pub enum DnsCommand {
  Set {
    /// Device name or ID.
    #[arg()]
    device: Option<String>,
    #[arg(long = "no-doh", default_value = "false")]
    no_doh: bool,
  },
}

#[derive(Subcommand, Debug)]
pub enum AdminCommand {
  Create,
  Delete,
  #[command(name = "change-password")]
  ChangePassword,
}
