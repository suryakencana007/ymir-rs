use axum::{response::Response, routing::get, Router};
use serde::Serialize;

use crate::{context::Context, render, Result};

pub fn register_handler(ctx: Context) -> Router {
    Router::new()
        .route("/healthz", get(ping))
        .route("/readyz", get(ping))
        .with_state(ctx)
}

#[derive(Serialize)]
struct Health {
    pub ok: bool,
}

async fn ping() -> Result<Response> {
    render::json(Health { ok: true })
}
