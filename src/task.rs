use anyhow::Result;

pub fn named_task<F, T>(name: &'static str, fut: F) -> impl Future<Output = Result<T>>
where
  F: Future<Output = Result<T>>,
{
  async move { fut.await.map_err(|e| anyhow::anyhow!("\ntask '{name}' failed: {e:#}\n")) }
}
