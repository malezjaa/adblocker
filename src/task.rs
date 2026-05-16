use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::error;

pub fn spawn_task<Fut>(
  name: &'static str,
  enabled: bool,
  f: Fut,
) -> Option<tokio::task::JoinHandle<()>>
where
  Fut: Future<Output = Result<()>> + Send + 'static,
{
  if !enabled {
    return None;
  }

  Some(tokio::spawn(async move {
    if let Err(err) = f.await {
      error!(error = ?err, "{name} failed");
    }
  }))
}