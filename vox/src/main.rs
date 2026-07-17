use anyhow::Result;
use clap::Parser;
use rustls::crypto::ring;
use tokio::{
  runtime::Runtime,
  sync::{mpsc::channel, oneshot::Receiver},
};
use vox::{application::app::App, context::Context};
use vox_shared::{SharedCli, logger::setup_logger};

#[cfg(windows)]
mod service;

fn main() -> Result<()> {
  let cli = SharedCli::parse();
  if cli.service {
    #[cfg(windows)]
    return service::dispatch();
    #[cfg(not(windows))]
    anyhow::bail!("--service is only supported on Windows");
  }

  setup_logger(cli.verbose);
  Runtime::new()?.block_on(run(None))
}

pub(crate) async fn run(shutdown: Option<Receiver<()>>) -> Result<()> {
  let (app, rx) = initialize().await?;
  app.start_all(rx, shutdown).await?;

  Ok(())
}

pub(crate) async fn initialize()
-> Result<(App, tokio::sync::mpsc::Receiver<vox::engine::EngineMessage>)> {
  ring::default_provider()
    .install_default()
    .expect("failed to install rustls crypto provider");

  let (tx, rx) = channel(100);
  let ctx = Context::new(tx).await?;
  ctx.load_mmdbs()?;

  let app = App::init(ctx).await?;
  Ok((app, rx))
}
