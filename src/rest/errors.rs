use argon2::password_hash::Error as ArgonError;
use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::rest::responses::JsonResponse;

pub enum ContextError {
    UnauthorizedAccess,
    InternalServerError,
    BadRequest,
    NotFound,
}

pub enum ErrorIssuer {
    JsonRejection(JsonRejection),
    PasswordHashError(ArgonError),
    UlidError(rusty_ulid::DecodingError),
    Unauthorized(String),
    InternalError(String),
    BadRequest(String),
    NotFound(String),
}

impl IntoResponse for ErrorIssuer {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
            status_code: u16,
        }

        let (status, message) = match self {
            ErrorIssuer::JsonRejection(rejection) => {
                tracing::error!("Bad user input: {:?}", rejection);
                (rejection.status(), rejection.body_text())
            }
            ErrorIssuer::PasswordHashError(error) => match error {
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
            ErrorIssuer::UlidError(error) => {
                tracing::error!("UUID error: {}", error);
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid UUID provided.".to_string(),
                )
            }
            ErrorIssuer::Unauthorized(error) => {
                tracing::warn!("Unauthorized access: {}", error);
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid Ulid provided.".to_string(),
                )
            }
            ErrorIssuer::InternalError(error) => {
                tracing::error!("Internal server error: {}", error);
                (StatusCode::INTERNAL_SERVER_ERROR, error)
            }
            ErrorIssuer::BadRequest(error) => {
                tracing::warn!("Bad request: {}", error);
                (StatusCode::BAD_REQUEST, error)
            }
            ErrorIssuer::NotFound(error) => {
                tracing::warn!("Not found: {}", error);
                (StatusCode::NOT_FOUND, error)
            }
        };

        (
            status,
            JsonResponse(ErrorResponse {
                message,
                status_code: status.as_u16(),
            }),
        )
            .into_response()
    }
}

impl From<JsonRejection> for ErrorIssuer {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonRejection(rejection)
    }
}

impl From<ArgonError> for ErrorIssuer {
    fn from(error: ArgonError) -> Self {
        Self::PasswordHashError(error)
    }
}

impl From<rusty_ulid::DecodingError> for ErrorIssuer {
    fn from(error: rusty_ulid::DecodingError) -> Self {
        Self::UlidError(error)
    }
}

impl From<(String, ContextError)> for ErrorIssuer {
    fn from((message, context): (String, ContextError)) -> Self {
        match context {
            ContextError::UnauthorizedAccess => ErrorIssuer::Unauthorized(message),
            ContextError::InternalServerError => ErrorIssuer::InternalError(message),
            ContextError::BadRequest => ErrorIssuer::BadRequest(message),
            ContextError::NotFound => ErrorIssuer::NotFound(message),
        }
    }
}
