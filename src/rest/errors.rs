use argon2::password_hash::Error as ArgonError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::rest::responses::Json;

pub enum ContextError {
    UnauthorizedAccess,
    InternalServerError,
    BadRequest,
    NotFound,
}

impl IntoResponse for crate::errors::Error {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
            status_code: u16,
        }

        let (status, message) = match self {
            Self::JsonRejection(rejection) => {
                tracing::error!("Bad user input: {:?}", rejection);
                (rejection.status(), rejection.body_text())
            }
            Self::PasswordHashError(error) => match error {
                ArgonError::Password => {
                    tracing::info!("Password mismatch error");
                    (
                        StatusCode::BAD_REQUEST,
                        "Email and Password combination does not match.".to_string(),
                    )
                }
                _ => {
                    tracing::error!("Password hashing error: {}", error);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An error occurred during password processing.".to_string(),
                    )
                }
            },
            Self::UlidError(error) => {
                tracing::error!("UUID error: {}", error);
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid UUID provided.".to_string(),
                )
            }
            Self::Unauthorized(error) => {
                tracing::warn!("Unauthorized access: {}", error);
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid Ulid provided.".to_string(),
                )
            }
            Self::InternalServerError(error) => {
                tracing::error!("Internal server error: {}", error);
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
            Self::BadRequest(error) => {
                tracing::warn!("Bad request: {}", error);
                (StatusCode::BAD_REQUEST, error.to_string())
            }
            Self::NotFound(error) => {
                tracing::warn!("Not found: {}", error);
                (StatusCode::NOT_FOUND, error)
            }
            _ => (StatusCode::BAD_REQUEST, "".to_string()),
        };

        (
            status,
            Json(ErrorResponse {
                message,
                status_code: status.as_u16(),
            }),
        )
            .into_response()
    }
}
