use std::{io::Result, time::Duration};
use tower_http::cors;

pub fn cors_middleware(cfg: &crate::settings::CorsMiddleware) -> Result<cors::CorsLayer> {
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
