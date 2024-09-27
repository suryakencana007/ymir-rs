use crate::{
    adapter::{QueueEngineAdapter, ReturnAdapter},
    hooks::{Context, LifeCycle},
    logo::print_logo,
    rest::ports,
    settings::{self, Environment},
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

    let settings = settings::get_settings(&environment).expect("Failed to read settings.");
    Ok(Context {
        key: axum_extra::extract::cookie::Key::from(settings.clone().secret.cookie.as_bytes()),
        environment: environment.clone(),
        settings,
        db: None,
        queue: None,
    })
}

pub async fn start<L: LifeCycle>() -> Result<()> {
    // create context
    let ctx = make_context().await?;
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
    println!("version: {}", L::version());

    // LifeCycle Adapters.
    let ReturnAdapter { ctx, adapters } = start_adapters::<L>(ctx.clone()).await?;

    // ports http serve
    ports::serve::<L>(ctx.clone()).await?;

    for adapter in &adapters {
        adapter.after_stop(ctx.clone()).await?;
    }
    tracing::info!(adapters = ?adapters.iter().map(|init| init.name()).collect::<Vec<_>>().join(","), "adapters stoped");
    Ok(())
}

pub async fn start_adapters<L: LifeCycle>(mut ctx: Context) -> Result<ReturnAdapter> {
    let mut adapters = L::adapters().await?;
    tracing::info!(adapters = ?adapters.iter().map(|init| init.name()).collect::<Vec<_>>().join(","), "adapters loaded");
    // register internal adapter
    adapters.push(Box::new(QueueEngineAdapter));
    for adapter in &adapters {
        ctx = adapter.before_run(ctx.clone()).await?;
    }
    Ok(ReturnAdapter { ctx, adapters })
}
