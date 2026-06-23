use anyhow::Result;

pub async fn named_task<F, T>(name: &'static str, fut: F) -> Result<T>
where
  F: Future<Output = Result<T>>,
{
  fut.await.map_err(|e| anyhow::anyhow!("\ntask '{name}' failed: {e:#}\n"))
}
