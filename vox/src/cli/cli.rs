use clap::{Parser, Subcommand, ValueEnum};
use vox::database::devices::DeviceType;

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(color = clap::ColorChoice::Always)]
#[command(styles = vox_shared::cli::styles())]
pub struct Cli {
  #[arg(short, long)]
  pub verbose: bool,

  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
  Service {
    #[command(subcommand)]
    command: ServiceCommand,
  },
  Devices {
    #[command(subcommand)]
    command: DeviceCommand,
  },
  Dns {
    #[command(subcommand)]
    command: DnsCommand,
  },
  Acme {
    #[command(subcommand)]
    command: AcmeCommand,
  },
  #[command(name = "reset-db")]
  ResetDB,

  Admin {
    #[command(subcommand)]
    command: AdminCommand,
  },
}

#[derive(Subcommand, Debug)]
pub enum ServiceCommand {
  Install {
    #[arg(value_enum)]
    target: ServiceTarget,
    /// Validate the bundle and print the actions without changing the machine.
    #[arg(long)]
    dry_run: bool,
  },
  Start {
    #[arg(value_enum)]
    target: ServiceTarget,
  },
  Stop {
    #[arg(value_enum)]
    target: ServiceTarget,
  },
  Restart {
    #[arg(value_enum)]
    target: ServiceTarget,
  },
  Status {
    #[arg(value_enum)]
    target: ServiceTarget,
  },
  Uninstall {
    #[arg(value_enum)]
    target: ServiceTarget,
    /// Also remove all machine-wide configuration and data.
    #[arg(long)]
    purge_data: bool,
  },
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceTarget {
  Daemon,
  Client,
}

#[derive(Subcommand, Debug)]
pub enum DeviceCommand {
  List,

  Create {
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
pub enum AcmeCommand {
  /// Complete the configured ACME DNS-01 challenge and fetch a certificate.
  Challenge,
}

#[derive(Subcommand, Debug)]
pub enum AdminCommand {
  Create,
  Delete,
  #[command(name = "change-password")]
  ChangePassword,
}
