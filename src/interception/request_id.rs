use std::sync::LazyLock;

use axum::{extract::Request, middleware::Next, response::Response};
use http::HeaderValue;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct RequestId(String);

impl RequestId {
    /// Get the request id
    #[must_use]
    pub fn get(&self) -> &str {
        self.0.as_str()
    }
}

const X_REQUEST_ID: &str = "x-request-id";
const MAX_LEN: usize = 255;

static ID_CLEANUP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^\w\-@]").unwrap());

pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let header_request_id = request.headers().get(X_REQUEST_ID).cloned();
    let request_id = make_request_id(header_request_id);
    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));
    let mut res = next.run(request).await;

    if let Ok(v) = HeaderValue::from_str(request_id.as_str()) {
        res.headers_mut().insert(X_REQUEST_ID, v);
    } else {
        tracing::warn!("could not set request ID into response headers: `{request_id}`",);
    }
    res
}

fn make_request_id(maybe_request_id: Option<HeaderValue>) -> String {
    maybe_request_id
        .and_then(|hdr| {
            let id: Option<String> = hdr.to_str().ok().map(|s| {
                ID_CLEANUP
                    .replace_all(s, "")
                    .chars()
                    .take(MAX_LEN)
                    .collect()
            });
            id.filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| rusty_ulid::Ulid::generate().to_string())
}
