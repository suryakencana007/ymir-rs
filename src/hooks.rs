use std::net::SocketAddr;

use async_trait::async_trait;
use axum::{extract::FromRef, Router};
use sea_orm::DatabaseConnection;

use crate::{
    adapter::Adapter,
    job::{Pool, Processor, RedisConnectionManager},
    rest::ports::shutdown_signal,
    settings::{Environment, Settings},
    Result,
};

#[derive(Clone)]
pub struct Context {
    /// Enables the PrivateCookieJar extractor
    pub key: axum_extra::extract::cookie::Key,
    /// The environment in which the application is running.
    pub environment: Environment,
    /// Settings for the application.
    pub settings: Settings,
    /// A database connection used by the application.
    pub db: Option<DatabaseConnection>,
    /// A worker tasks
    pub queue: Option<Pool<RedisConnectionManager>>,
}

impl FromRef<Context> for axum_extra::extract::cookie::Key {
    fn from_ref(state: &Context) -> Self {
        state.key.clone()
    }
}

#[async_trait]
pub trait LifeCycle {
    #[must_use]
    fn version() -> String {
        "dev".to_string()
    }
    /// Defines the crate name
    ///
    /// Example
    /// ```rust
    /// fn app_name() -> &'static str {
    ///     env!("CARGO_CRATE_NAME")
    /// }
    /// ```
    fn app_name() -> &'static str;

    /// Start serving the Axum web application on the specified address and
    /// port.
    ///
    /// # Returns
    /// A Result indicating success () or an error if the server fails to start.
    async fn rest(ctx: &Context, app: Router) -> Result<()> {
        let settings = ctx.settings.clone();
        let address = format!("{}:{}", settings.server.host, settings.server.port);

        let listener = tokio::net::TcpListener::bind(&address).await?;

        tracing::info!("Listening on {}", &address);

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await?;
        Ok(())
    }

    /// Register external adapters to the application.
    async fn adapters() -> Result<Vec<Box<dyn Adapter>>> {
        Ok(vec![])
    }

    /// Run all custom job runner.
    fn job_runners<'a>(p: &'a mut Processor, ctx: &'a Context);

    /// Router
    fn routes(ctx: Context) -> Router;
}
