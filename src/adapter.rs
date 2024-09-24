use crate::{hooks::Context, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Adapter: Sync + Send {
    /// The Adapter name
    fn name(&self) -> String;

    async fn before_run(&self, ctx: Context) -> Result<Context> {
        Ok(ctx)
    }

    async fn after_stop(&self, _ctx: Context) -> Result<()> {
        Ok(())
    }
}
