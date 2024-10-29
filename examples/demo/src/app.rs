use async_trait::async_trait;
use axum::Router;
use ymir::{adapter::Adapter, context::Context, hooks::LifeCycle, Result};
use ymir_openapi::prelude::*;

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
        RouterDoc::new()
            .build_doc("/api/swagger", |mut doc| {
                doc.info = openapi::Info::new(
                    "Demo Ymir API DOC",
                    &format!("v{}", env!("CARGO_PKG_VERSION")),
                );

                doc.info.description = Some("Demo YMIR OPENAPI UI".to_string());
                doc
            })
            .routes(routes!(health))
            .route(
                "/api/health-check-one",
                axum::routing::get(|| async { "OK" }),
            )
            .with_state(ctx)
    }
}

#[utoipa::path(
    get,
    path = "/health",
     tag = "api",
    responses(
        (status = OK, description = "Success", body = str)
    )
)]
pub async fn health() -> &'static str {
    "OK"
}
