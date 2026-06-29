use anyhow::Result;
use clap::Parser;
use rustls::crypto::ring;
use scopeguard::defer;
use std::sync::Arc;
use tokio::sync::mpsc::channel;
use vox::application::app::App;
use vox::context::Context;
use vox_shared::logger::setup_logger;

#[derive(Parser, Debug)]
struct Cli {
  #[arg(short, long)]
  verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
  ring::default_provider()
    .install_default()
    .expect("failed to install rustls crypto provider");
  let cli = Cli::parse();
  setup_logger(cli.verbose);

  let (tx, rx) = channel(100);
  let ctx = Context::new(tx).await?;
  ctx.load_mmdbs()?;

  let app = Arc::new(App::init(ctx).await?);
  defer! {
    #[cfg(windows)] unsafe {
      use ::windows::Win32::NetworkManagement::WindowsFilteringPlatform::FwpmEngineClose0;
      FwpmEngineClose0(app.wfp_sess.engine);
    }
  }

  app.start_all(rx).await?;

  Ok(())
}
