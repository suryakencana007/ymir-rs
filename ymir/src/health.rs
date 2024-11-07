use axum::response::Response;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{render, Result};

#[derive(Serialize, ToSchema)]
pub struct Health {
    pub ok: bool,
}

#[utoipa::path(
    get,
    path = "/healthz",
    tag = "status",
    responses(
        (status = OK, description = "Success", body = Health),
        (status = 400, description = "Bad Request", body = crate::errors::ErrorResponse)
    )
)]
pub async fn healthz() -> Result<Response> {
    render::json(Health { ok: true })
}

#[utoipa::path(
    get,
    path = "/readyz",
    tag = "status",
    responses(
        (status = OK, description = "Success", body = Health),
        (status = 400, description = "Bad Request", body = crate::errors::ErrorResponse)
    )
)]
pub async fn readyz() -> Result<Response> {
    render::json(Health { ok: true })
}
