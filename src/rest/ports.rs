use std::path::PathBuf;

use axum::Router;
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};

use crate::{
    errors,
    hooks::{Context, LifeCycle},
    Result,
};

use super::{health, middlewares::serve_http};

pub async fn serve<L: LifeCycle>(ctx: &Context, router: Router) -> Result<()> {
    L::rest(ctx, router).await
}

pub async fn build_routes<L: LifeCycle>(ctx: &Context) -> Result<Router> {
    let settings = ctx.settings.clone();
    // build our application with a route
    let mut app = axum::Router::new()
        .merge(L::routes(ctx.clone()))
        .merge(health::register_handler(ctx.clone()))
        .layer(tower_http::trace::TraceLayer::new_for_http());
    app = serve_http(ctx.clone(), app.clone());

    // Static Assets
    if let Some(assets) = settings
        .server
        .middlewares
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
    // L::rest(ctx.clone(), app).await?;
    Ok(app)
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
