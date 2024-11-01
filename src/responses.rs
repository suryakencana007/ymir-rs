use std::borrow::Cow;

use axum::{
    extract::FromRequest,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use serde::Serialize;
use utoipa::{PartialSchema, ToSchema};

use crate::errors::Error;

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(Error))]
pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

#[derive(Serialize, ToSchema)]
pub struct Success {
    pub message: String,
    pub status_code: u16,
}

#[derive(Serialize)]
pub struct UlidSchema(pub rusty_ulid::Ulid);

impl PartialSchema for UlidSchema {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::schema::Object::builder()
            .schema_type(utoipa::openapi::schema::SchemaType::AnyValue)
            .into()
    }
}

impl ToSchema for UlidSchema {
    fn name() -> std::borrow::Cow<'static, str> {
        Cow::Borrowed("UlidSchema")
    }
}

impl Default for Success {
    fn default() -> Self {
        Self {
            message: "Success".to_string(),
            status_code: StatusCode::OK.as_u16(),
        }
    }
}

impl IntoResponse for Success {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::OK);
        let json_body = axum::Json(self);

        // Convert Json to Response
        let mut response = json_body.into_response();

        // Set the correct status code
        *response.status_mut() = status;

        response
    }
}
