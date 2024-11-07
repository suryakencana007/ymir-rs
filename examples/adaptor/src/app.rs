use async_trait::async_trait;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Router,
};
use ymir::{
    adapter::Adapter, context::Context, health, hook::LifeCycle, responses::Success, Result,
};
use ymir_openapi::prelude::*;

use crate::adapters::metrics::MetricsAdapter;

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
        Ok(vec![Box::new(MetricsAdapter::new("/metrics".to_string()))])
    }

    fn routes(ctx: Context) -> Router {
        let kunci = Kunci {
            label: String::from("kunci itu ada disini"),
        };
        RouterDoc::new()
            .build_doc("/api/swagger", |mut doc| {
                doc.info = openapi::Info::new(
                    "Metric Adapter",
                    &format!("v{}", env!("CARGO_PKG_VERSION")),
                );

                doc.info.description = Some("Demo YMIR OPENAPI UI".to_string());
                doc
            })
            .routes(routes!(health::healthz))
            .routes(routes!(health))
            .route(
                "/api/health-check-one",
                axum::routing::get(|| async { "OK" }),
            )
            .layer(Extension(kunci))
            .with_state(ctx)
    }
}

#[utoipa::path(
    get,
    path = "/health",
     tag = "api",
    responses(
        (status = OK, description = "Success", body = ymir::responses::Success),
        (status = 400, description = "Bad Request", body = ymir::errors::ErrorResponse)
    )
)]
pub async fn health(Extension(k): Extension<Kunci>) -> Result<Response> {
    Ok(Success {
        message: k.label.to_string(),
        status_code: StatusCode::OK.as_u16(),
    }
    .into_response())
}

#[derive(Clone)]
pub struct Kunci {
    pub label: String,
}
