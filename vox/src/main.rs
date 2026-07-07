use anyhow::Result;
use clap::Parser;
use rustls::crypto::ring;
use tokio::sync::mpsc::channel;
use vox::application::app::App;
use vox::context::Context;
use vox_shared::SharedCli;
use vox_shared::logger::setup_logger;

#[tokio::main]
async fn main() -> Result<()> {
  ring::default_provider()
    .install_default()
    .expect("failed to install rustls crypto provider");
  let cli = SharedCli::parse();
  setup_logger(cli.verbose);

  let (tx, rx) = channel(100);
  let ctx = Context::new(tx).await?;
  ctx.load_mmdbs()?;

  let app = App::init(ctx).await?;

  app.start_all(rx).await?;

  Ok(())
}
