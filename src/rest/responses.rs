use axum::{
    extract::FromRequest,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rusty_ulid::Ulid;
use serde::Serialize;

use crate::errors::Error;

#[derive(Debug, FromRequest)]
#[from_request(via(axum::Json), rejection(Error))]
pub struct Json<T>(pub T);

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

#[derive(Serialize)]
pub struct Success {
    pub message: String,
    pub status_code: u16,
    pub user_id: Option<Ulid>,
}

impl Default for Success {
    fn default() -> Self {
        Self {
            message: "Success".to_string(),
            status_code: StatusCode::OK.as_u16(),
            user_id: None,
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
