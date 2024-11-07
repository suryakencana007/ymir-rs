pub mod request_id;

use std::{sync::LazyLock, time::Duration};

use axum::{response::IntoResponse, Router};
use request_id::request_id_middleware;
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer, cors,
    set_header::SetResponseHeaderLayer, timeout::TimeoutLayer,
};

use crate::{config::Environment, context::Context, errors::Error, Result};

static DEFAULT_IDENT_HEADER_NAME: LazyLock<http::header::HeaderName> =
    LazyLock::new(|| http::header::HeaderName::from_static("x-powered-by"));

static DEFAULT_IDENT_HEADER_VALUE: LazyLock<http::header::HeaderValue> =
    LazyLock::new(|| http::header::HeaderValue::from_static("butter"));

pub fn interception_fn(ctx: Context, mut router: Router) -> Router {
    let cfg = ctx.configs.clone().expect("load configuration failed.");

    // CORS Middleware
    if let Some(cors) = cfg.server.interceptions.cors.as_ref().filter(|c| c.enable) {
        router = router.layer(tower::ServiceBuilder::new().layer(interception_cors(cors).unwrap()));
        tracing::info!("[Middleware] +cors");
    }
    // Timeout Middleware
    if let Some(timeout) = cfg
        .server
        .interceptions
        .timeout_request
        .as_ref()
        .filter(|c| c.enable)
    {
        router = router.layer(TimeoutLayer::new(Duration::from_millis(timeout.timeout)));
        tracing::info!("[Middleware] +timeout");
    }

    // Compression Middleware
    if cfg
        .server
        .interceptions
        .compression
        .as_ref()
        .filter(|c| c.enable)
        .is_some()
    {
        router = router.layer(CompressionLayer::new());
        tracing::info!("[Middleware] +compression");
    }

    // Limit Payload
    if let Some(limit) = cfg
        .server
        .interceptions
        .limit_payload
        .as_ref()
        .filter(|c| c.enable)
    {
        router = router.layer(axum::extract::DefaultBodyLimit::max(
            byte_unit::Byte::parse_str(&limit.body_limit, false)
                .unwrap()
                .as_u128() as usize,
        ));
        tracing::info!(data = &limit.body_limit, "[Middleware] +limit payload");
    }

    // catch panic
    match ctx.environment.unwrap() {
        Environment::Development => {
            router = router.layer(CatchPanicLayer::custom(handle_panic));
        }
        // TODO! Production Env.
        Environment::Production => (),
    }

    router = router.layer(SetResponseHeaderLayer::overriding(
        DEFAULT_IDENT_HEADER_NAME.clone(),
        DEFAULT_IDENT_HEADER_VALUE.clone(),
    ));

    router = router.layer(axum::middleware::from_fn(request_id_middleware));

    router
}

pub fn interception_cors(cfg: &crate::config::InterceptionCors) -> Result<cors::CorsLayer> {
    let mut cors: cors::CorsLayer = cors::CorsLayer::permissive();

    if let Some(allow_origins) = &cfg.allow_origins {
        let mut list = vec![];
        for origins in allow_origins {
            list.push(origins.parse::<axum::http::HeaderValue>().unwrap());
        }
        cors = cors.allow_origin(list);
    }

    if let Some(allow_headers) = &cfg.allow_headers {
        let mut headers = vec![];
        for header in allow_headers {
            headers.push(header.parse().unwrap());
        }
        cors = cors.allow_headers(headers);
    }

    if let Some(allow_methods) = &cfg.allow_methods {
        let mut methods = vec![];
        for method in allow_methods {
            methods.push(method.parse().unwrap());
        }
        cors = cors.allow_methods(methods);
    }

    if let Some(max_age) = cfg.max_age {
        cors = cors.max_age(Duration::from_secs(max_age));
    }
    Ok(cors)
}

/// Handler function for the [`CatchPanicLayer`] middleware.
#[allow(clippy::needless_pass_by_value)]
fn handle_panic(err: Box<dyn std::any::Any + Send + 'static>) -> axum::response::Response {
    let err = err.downcast_ref::<String>().map_or_else(
        || err.downcast_ref::<&str>().map_or("no error details", |s| s),
        |s| s.as_str(),
    );

    tracing::error!(err.msg = err, "server_panic");

    Error::InternalServerError(err.to_string()).into_response()
}
