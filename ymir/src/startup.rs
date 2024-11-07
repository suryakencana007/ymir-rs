use std::path::PathBuf;

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    adapter::AdapterManager,
    config::{load_configuration, Environment},
    context::Context,
    errors::{self, Error},
    hook::LifeCycle,
    interception::interception_fn,
    logo::print_logo,
    Result,
};

const MODULE_WHITELIST: &[&str] = &[
    "ymir",
    "ymir-openapi",
    "tower_http",
    "axum::rejection",
    "sqlx",
];

/// Create context application.
pub async fn create_context() -> Result<Context> {
    dotenvy::dotenv_override().ok();
    // Detect the running environment.
    // Default to `development` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let configs = load_configuration(&environment).expect("Failed to read configurations.");

    Ok(Context {
        environment: Some(environment.clone()),
        configs: Some(configs),
        extend: Some(Box::default()),
    })
}

/// Create axum router.
pub async fn router_init<LC: LifeCycle>(ctx: &Context) -> Result<Router> {
    let config = ctx.configs.clone().expect("load configuration failed.");
    // build our application with a route
    let mut app = axum::Router::new()
        .merge(LC::routes(ctx.clone()))
        // .merge(health::register_handler(ctx.clone()))
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

/// Run the impl app struct to the application.
pub async fn run<LC: LifeCycle>() -> Result<()> {
    let ctx = create_context().await.expect("create context is failed.");
    let conf = ctx.configs.clone().expect("load configuration failed.");
    let logger = conf.logger.clone();
    let level = logger
        .enable
        .then(|| logger.level)
        .or_else(|| Some("error".to_string()))
        .unwrap();

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| {
            tracing_subscriber::EnvFilter::try_new(
                MODULE_WHITELIST
                    .iter()
                    .map(|m| format!("{m}={level}"))
                    .chain(std::iter::once(format!("{}={}", LC::app_name(), level)))
                    .collect::<Vec<_>>()
                    .join(","),
            )
        })
        .expect("tracing filter failed");
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    print_logo(ctx.environment.clone().unwrap(), conf.clone());
    println!("version: {}", LC::version());

    let mut adapter_manager = AdapterManager::new(ctx);
    let adapters = LC::adapters().await?;
    for adapter in adapters {
        adapter_manager.register(adapter);
    }
    adapter_manager.init_all().await?;
    let ctx = adapter_manager.before_run().await?;
    let router = router_init::<LC>(&ctx).await?;
    let app = adapter_manager.configure_routes(router).await?;

    #[cfg(test)]
    {
        use std::{
            sync::{Arc, Mutex},
            time::Duration,
        };

        // Use a mocked signal for testing
        let mock_shutdown_signal =
            Arc::new(Mutex::new(Some(tokio::sync::oneshot::channel::<()>())));
        let mock_signal_receiver = mock_shutdown_signal.lock().unwrap().take().unwrap().1;
        let shutdown_signal = async {
            tokio::time::sleep(Duration::from_secs(3)).await;
            let _ = mock_signal_receiver.await;
        };
        tokio::select! {
            result = LC::rest(&ctx, app) => {
                if let Err(err) = result {
                    return Err(Error::Message(err.to_string()));
                }
            }
            _ = shutdown_signal => {
                println!("Received shutdown signal");
            }
        }
    }
    #[cfg(not(test))]
    {
        if let Err(err) = LC::rest(&ctx, app).await {
            return Err(Error::Message(err.to_string()));
        }
    }

    let mut rx = adapter_manager.shutdown_signal();
    tokio::spawn(async move {
        adapter_manager.stop_all().await.unwrap();
    });

    match rx.recv().await {
        Ok(()) => Ok(()),
        Err(e) => Err(Error::Message(format!("{}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::Adapter;
    use async_trait::async_trait;
    use axum::{body::Body, extract::Request};
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };
    use tokio::sync::oneshot;
    use tower::ServiceExt; // for `oneshot` method

    #[tokio::test]
    async fn test_create_context() {
        let ctx = create_context().await.unwrap();
        assert!(ctx.environment.is_some());
        assert!(ctx.configs.is_some());
        assert!(ctx.extend.is_some());
    }

    #[tokio::test]
    async fn test_router_init() {
        let ctx = create_context().await.unwrap();
        let router = router_init::<MockLifeCycle>(&ctx)
            .await
            .expect("failed to create router");
        assert!(router.has_routes());

        // Testing the router
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("Failed to execute request");
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_run() {
        // Create a mock shutdown signal for testing
        let mock_shutdown_signal = Arc::new(Mutex::new(Some(oneshot::channel::<()>())));
        let run_handle = tokio::spawn(async move { run::<MockLifeCycle>().await });
        // Simulate sending a shutdown signal
        {
            let shutdown_signal = mock_shutdown_signal.lock().unwrap().take().unwrap().0;
            let _ = shutdown_signal.send(());
        }
        let result = tokio::time::timeout(Duration::from_secs(5), run_handle).await;
        match result {
            Ok(Ok(_)) => assert!(true),
            Ok(Err(e)) => panic!("run function failed: {}", e),
            Err(_) => panic!("run function timed out"),
        }
    }

    struct MockLifeCycle;

    #[async_trait]
    impl LifeCycle for MockLifeCycle {
        fn app_name() -> &'static str {
            "test-app"
        }

        fn version() -> String {
            "0.1.0".to_string()
        }

        fn routes(ctx: Context) -> Router {
            Router::new()
                .route("/health", axum::routing::get(|| async { "OK" }))
                .with_state(ctx)
        }

        async fn adapters() -> Result<Vec<Box<dyn Adapter>>> {
            Ok(vec![])
        }
    }
}
