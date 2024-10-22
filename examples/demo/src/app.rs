use async_trait::async_trait;
use axum::Router;
use ymir::{adapter::Adapter, context::Context, hooks::LifeCycle, Result};

pub struct App;
#[async_trait]
impl LifeCycle for App {
    fn version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    async fn adapters() -> Result<Vec<Box<dyn Adapter>>> {
        Ok(vec![])
    }

    fn routes(ctx: Context) -> Router {
        Router::new()
            .route(
                "/api/health-check-one",
                axum::routing::get(|| async { "OK" }),
            )
            .with_state(ctx)
    }
}
