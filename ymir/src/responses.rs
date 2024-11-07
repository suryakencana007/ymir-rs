use axum::{
    extract::FromRequest,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

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
        (self.status(), axum::Json(self).into_response()).into_response()
    }
}

#[derive(Serialize, ToSchema)]
pub struct Success {
    pub message: String,
    pub status_code: u16,
}

impl Success {
    pub fn status(&self) -> StatusCode {
        StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
