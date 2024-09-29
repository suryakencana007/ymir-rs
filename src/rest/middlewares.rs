use std::time::Duration;

use axum::{http, Router};
use lazy_static::lazy_static;
use tower_http::{
    compression::CompressionLayer, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer,
};

use crate::{
    hooks::Context,
    rest::{cors::cors_middleware, request_id::request_id_middleware},
};

lazy_static! {
    static ref DEFAULT_IDENT_HEADER_NAME: http::header::HeaderName =
        http::header::HeaderName::from_static("x-powered-by");
    static ref DEFAULT_IDENT_HEADER_VALUE: http::header::HeaderValue =
        http::header::HeaderValue::from_static("butter");
}

pub fn serve_http(ctx: Context, mut router: Router) -> Router {
    let settings = ctx.settings.clone();
    // CORS Middleware
    if let Some(cors) = settings
        .server
        .middlewares
        .cors
        .as_ref()
        .filter(|c| c.enable)
    {
        router = router.layer(tower::ServiceBuilder::new().layer(cors_middleware(cors).unwrap()));
        tracing::info!("[Middleware] +cors");
    }
    // Timeout Middleware
    if let Some(timeout) = settings
        .server
        .middlewares
        .timeout_request
        .as_ref()
        .filter(|c| c.enable)
    {
        router = router.layer(TimeoutLayer::new(Duration::from_millis(timeout.timeout)));
        tracing::info!("[Middleware] +timeout");
    }

    // Compression Middleware
    if settings
        .server
        .middlewares
        .compression
        .as_ref()
        .filter(|c| c.enable)
        .is_some()
    {
        router = router.layer(CompressionLayer::new());
        tracing::info!("[Middleware] +compression");
    }

    // Limit Payload
    if let Some(limit) = settings
        .server
        .middlewares
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

    router = router.layer(SetResponseHeaderLayer::overriding(
        DEFAULT_IDENT_HEADER_NAME.clone(),
        DEFAULT_IDENT_HEADER_VALUE.clone(),
    ));

    router = router.layer(axum::middleware::from_fn(request_id_middleware));

    router
}
