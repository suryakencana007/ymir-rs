use crate::{
    hooks::Context,
    job::{Pool, RedisConnectionManager},
    Result,
};
use async_trait::async_trait;

pub struct ReturnAdapter {
    pub ctx: Context,
    pub adapters: Vec<Box<dyn Adapter>>,
}

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

/// Redis and Sidekiq for Queue adapter.
pub struct QueueEngineAdapter;
#[async_trait]
impl Adapter for QueueEngineAdapter {
    fn name(&self) -> String {
        "queue-engine".to_string()
    }

    async fn before_run(&self, mut ctx: Context) -> Result<Context> {
        if let Some(redis) = &ctx.settings.cache {
            let manager = RedisConnectionManager::new(redis.uri.clone())
                .map_err(|err| format!("Redis connection is failed: {:?}", err))
                .unwrap();
            let pool = Pool::builder().build(manager).await.unwrap();
            ctx = Context {
                queue: Some(pool),
                ..ctx
            }
        } else {
            tracing::warn!("Queue engine is not found")
        }
        Ok(ctx)
    }
}
