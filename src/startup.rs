use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    adapter::ReturnAdapter,
    config::{load_configuration, Environment},
    context::Context,
    hooks::{self, LifeCycle},
    logo::print_logo,
    Result,
};

const MODULE_WHITELIST: &[&str] = &["ymir", "tower_http", "axum::rejection", "sqlx"];

pub async fn make_context() -> Result<Context> {
    dotenvy::dotenv_override().ok();

    // Detect the running environment.
    // Default to `development` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let configs = load_configuration(&environment).expect("Failed to read configurations.");

    Ok(Context {
        environment: environment.clone(),
        configs,
    })
}

pub async fn start_adapters<L: LifeCycle>(mut ctx: Context) -> Result<ReturnAdapter> {
    let adapters = L::adapters().await?;
    tracing::info!(adapters = ?adapters.iter().map(|init| init.name()).collect::<Vec<_>>().join(","), "adapters loaded");
    for adapter in &adapters {
        ctx = adapter.before_run(ctx).await?;
    }
    Ok(ReturnAdapter { ctx, adapters })
}

pub async fn start<L: LifeCycle>() -> Result<()> {
    // create context
    let ctx = make_context().await?;
    let logger = ctx.configs.logger.clone();
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
                    .chain(std::iter::once(format!("{}={}", L::app_name(), level)))
                    .collect::<Vec<_>>()
                    .join(","),
            )
        })
        .expect("tracing filter failed");
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    print_logo(ctx.environment.clone(), ctx.configs.clone());
    println!("version: {}", L::version());

    // LifeCycle Adapters.
    let ReturnAdapter { ctx, adapters } = start_adapters::<L>(ctx.clone()).await?;

    // make router
    let mut router = hooks::build_routes::<L>(&ctx).await?;
    for adapter in &adapters {
        router = adapter.after_route(&ctx, router).await?;
    }
    // ports http serve
    hooks::serve::<L>(&ctx, router).await?;
    for adapter in &adapters {
        adapter.after_stop(ctx.clone()).await?;
    }
    tracing::info!(adapters = ?adapters.iter().map(|init| init.name()).collect::<Vec<_>>().join(","), "adapters stoped");
    Ok(())
}
