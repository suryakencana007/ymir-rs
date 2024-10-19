use crate::{context::Context, Result};
use async_trait::async_trait;
use axum::Router;

#[async_trait]
pub trait Adapter: Sync + Send {
    /// The Adapter name
    fn name(&self) -> String;

    async fn before_run(&self, ctx: Context) -> Result<Context> {
        Ok(ctx)
    }

    async fn after_route(&self, _ctx: &Context, router: Router) -> Result<Router> {
        Ok(router)
    }

    async fn after_stop(&self, _ctx: Context) -> Result<()> {
        Ok(())
    }
}

pub struct ReturnAdapter {
    pub ctx: Context,
    pub adapters: Vec<Box<dyn Adapter>>,
}
