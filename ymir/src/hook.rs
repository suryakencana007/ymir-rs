use std::net::SocketAddr;

use async_trait::async_trait;
use axum::Router;

use crate::{adapter::Adapter, context::Context, errors::Error, signal::shutdown_signal, Result};

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
        let settings = ctx.configs.clone().expect("load configuration failed.");
        let address = format!("{}:{}", settings.server.host, settings.server.port);

        let listener = tokio::net::TcpListener::bind(&address).await?;

        tracing::info!("Listening on {}", &address);

        match axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::Message(format!("{}", e))),
        }
    }

    /// Register external adapters to the application.
    async fn adapters() -> Result<Vec<Box<dyn Adapter>>> {
        Ok(vec![])
    }

    /// Router
    fn routes(ctx: Context) -> Router;
}
