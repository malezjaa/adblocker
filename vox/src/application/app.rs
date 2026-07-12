use crate::certs::crl::serve_crl_pem;
use crate::context::Context;
use crate::database::DB;
use crate::engine::{EngineActor, EngineMessage};
use anyhow::Result;
use chrono::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::task::{JoinSet, LocalSet};
use tracing::error;
use tracing::log::warn;
use vox_shared::config::certs::CertificateStrategy;
use vox_shared::task::named_task;
#[cfg(windows)]
use vox_windows::WfpSession;
#[cfg(windows)]
use vox_windows::open_port::{OpenPortsConfig, open_ports};

pub struct App {
  #[cfg(windows)]
  pub wfp_sess: Option<WfpSession>,
  pub ctx: Context,
}

impl App {
  #[cfg(windows)]
  pub async fn init(ctx: Context) -> Result<Self> {
    let config = ctx.config();
    let wfp_sess = if config.firewall.open_ports {
      let ports =
        OpenPortsConfig { dns_port: config.dns.port, doh_port: config.doh.port };
      Some(open_ports(ports)?)
    } else {
      None
    };
    drop(config);

    Ok(Self { ctx, wfp_sess })
  }

  #[cfg(not(windows))]
  pub async fn init(ctx: Context) -> Result<Self> {
    Ok(Self { ctx })
  }

  pub async fn start_all(&self, rx: Receiver<EngineMessage>) -> Result<()> {
    let local = LocalSet::new();

    local
      .run_until(async {
        let config = self.ctx.config();
        self.ctx.spawn_config_watcher()?;

        let mut tasks = JoinSet::new();
        let engine = EngineActor::new(self.ctx.clone()).await?;

        tasks.spawn_local(named_task("AdBlocking engine", async move {
          let mut engine = engine;
          engine.run(rx).await
        }));
        tasks.spawn(named_task(
          "DB cleanup",
          DB::spawn_cleanup_task(self.ctx.db().pool.clone(), Duration::days(30)),
        ));

        if config.dns.enabled {
          tasks.spawn(named_task("DNS", Self::start_dns(self.ctx.clone())));
        }

        if matches!(config.certs.strategy, CertificateStrategy::SelfSigned) {
          tasks.spawn(named_task("CRL server", serve_crl_pem()));
        }

        if config.doh.enabled {
          tasks.spawn(named_task("DoH", Self::start_doh(self.ctx.clone())));
        }
        if config.dashboard {
          tasks.spawn(named_task("Dashboard", Self::start_dashboard(self.ctx.clone())));
        }
        // mutex guard would be held across await point below
        drop(config);

        while let Some(result) = tasks.join_next().await {
          match result {
            Ok(Ok(())) => {
              warn!("a background task exited unexpectedly");
            }
            Ok(Err(err)) => {
              error!("{:?}", err);
            }
            Err(err) => {
              if err.is_cancelled() {
                warn!("a background task was cancelled");
              } else if err.is_panic() {
                error!("a background task panicked: {:?}", err);
              } else {
                error!("task join error: {:?}", err);
              }
            }
          }
        }

        Ok(())
      })
      .await
  }
}
