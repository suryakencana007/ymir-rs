use std::{net::SocketAddr, path::PathBuf};

use async_trait::async_trait;
use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

use crate::{
    adapter::Adapter, context::Context, errors, health, interception::interception_fn,
    signal::shutdown_signal, Result,
};

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
        let settings = ctx.configs.clone();
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

    /// Router
    fn routes(ctx: Context) -> Router;
}

pub async fn serve<L: LifeCycle>(ctx: &Context, router: Router) -> Result<()> {
    L::rest(ctx, router).await
}

pub async fn build_routes<L: LifeCycle>(ctx: &Context) -> Result<Router> {
    let config = ctx.configs.clone();
    // build our application with a route
    let mut app = axum::Router::new()
        .merge(L::routes(ctx.clone()))
        .merge(health::register_handler(ctx.clone()))
        .layer(tower_http::trace::TraceLayer::new_for_http());
    app = interception_fn(ctx.clone(), app.clone());

    // Static Assets
    if let Some(assets) = config
        .server
        .interceptions
        .static_assets
        .as_ref()
        .filter(|c| c.enable)
    {
        if assets.must_exist
            && (!PathBuf::from(&assets.folder.path).exists()
                || !PathBuf::from(&assets.fallback).exists())
        {
            return Err(errors::Error::Message(format!(
                "static path are not found, Folder {} fallback: {}",
                assets.folder.path, assets.fallback
            )));
        }
        tracing::info!("[Middleware] +static assets");
        let serve_dir =
            ServeDir::new(&assets.folder.path).not_found_service(ServeFile::new(&assets.fallback));
        app = app.nest_service(
            &assets.folder.uri,
            if assets.precompressed {
                tracing::info!("[Middleware] +precompressed static assets");
                serve_dir.precompressed_gzip()
            } else {
                serve_dir
            },
        )
    }
    Ok(app)
}
