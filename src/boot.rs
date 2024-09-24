use crate::{
    hooks::{Context, LifeCycle},
    logo::print_logo,
    rest::ports,
    settings::{get_settings, Environment},
    Result,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const MODULE_WHITELIST: &[&str] = &["ymir", "tower_http", "axum::rejection", "sqlx"];

pub async fn make_context() -> Result<Context> {
    dotenv::dotenv().ok();

    // Detect the running environment.
    // Default to `development` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let settings = get_settings(&environment).expect("Failed to read settings.");
    Ok(Context {
        key: axum_extra::extract::cookie::Key::from(settings.clone().secret.cookie.as_bytes()),
        environment: environment.clone(),
        settings,
        db: None,
    })
}

pub async fn start<L: LifeCycle>() -> Result<()> {
    // create context
    let mut ctx = make_context().await?;
    let logger = ctx.settings.logger.clone();
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

    print_logo(ctx.environment.clone(), ctx.settings.clone());
    println!("{} {}", L::version(), ctx.settings.database.uri);

    // LifeCycle Adapters.
    let adapters = L::adapters().await?;
    tracing::info!(adapters = ?adapters.iter().map(|init| init.name()).collect::<Vec<_>>().join(","), "adapters loaded");
    for adapter in &adapters {
        ctx = adapter.before_run(ctx.clone()).await?;
    }

    // ports http serve
    ports::serve::<L>(ctx.clone()).await?;

    for adapter in &adapters {
        adapter.after_stop(ctx.clone()).await?;
    }
    tracing::info!(adapters = ?adapters.iter().map(|init| init.name()).collect::<Vec<_>>().join(","), "adapters stoped");
    Ok(())
}
