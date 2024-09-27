use async_trait::async_trait;
pub use bb8::Pool;
pub use sidekiq::{Processor, RedisConnectionManager, Result, Worker};
use tracing::error;

use crate::hooks::Context;

#[async_trait]
pub trait Jobs<T>: Worker<T>
where
    Self: Sized,
    T: Send + Sync + serde::Serialize + 'static,
{
    fn build(ctx: &Context) -> Self;
    async fn publish_job(ctx: &Context, args: T) -> Result<()> {
        if let Some(queue) = &ctx.queue {
            Self::perform_async(queue, args).await.unwrap();
        } else {
            error!(
                error.msg = "runner mode requested but no queue connection supplied, skipping job",
                "runner_error"
            );
        }
        Ok(())
    }
}
