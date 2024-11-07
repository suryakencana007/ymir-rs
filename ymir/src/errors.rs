use axum::{
    extract::rejection::JsonRejection,
    http::{
        header::{InvalidHeaderName, InvalidHeaderValue},
        method::InvalidMethod,
    },
    response::{IntoResponse, Response},
};
use http::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

use crate::responses::Json;

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::JSON(error)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Message(String),

    #[error(transparent)]
    Axum(#[from] axum::http::Error),

    #[error(transparent)]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error(transparent)]
    UlidError(#[from] ulid::DecodeError),

    #[error(transparent)]
    JSON(serde_json::Error),

    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    // API
    #[error("{0}")]
    Unauthorized(String),

    // API
    #[error("{0}")]
    NotFound(String),

    #[error("")]
    CustomError(StatusCode, String),

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    InternalServerError(String),

    #[error(transparent)]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[error(transparent)]
    InvalidHeaderName(#[from] InvalidHeaderName),

    #[error(transparent)]
    InvalidMethod(#[from] InvalidMethod),

    #[error(transparent)]
    Any(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub fn wrap(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Any(Box::new(err))
    }

    pub fn msg(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Message(err.to_string())
    }
    #[must_use]
    pub fn string(s: &str) -> Self {
        Self::Message(s.to_string())
    }
}

#[derive(Serialize, ToSchema)]
/// Structure representing details about an error.
pub struct ErrorResponse {
    message: String,
    status_code: u16,
}

impl ErrorResponse {
    #[must_use]
    pub fn new<T: Into<String>>(code: StatusCode, message: T) -> Self {
        Self {
            message: message.into(),
            status_code: code.as_u16(),
        }
    }
}
/// ```rust
/// use axum::response::Response;
/// use ymir::errors::Error;
///
/// async fn get_hello() -> Result<Response, Error> {
///     Err(Error::NotFound("ok".to_string()))
/// }
/// ````
impl IntoResponse for Error {
    /// Convert an `Error` into an HTTP response.
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound(error) => {
                tracing::error!("Not Found: {}", error);
                (
                    StatusCode::NOT_FOUND,
                    ErrorResponse::new(StatusCode::NOT_FOUND, error),
                )
            }
            Self::InternalServerError(error) => {
                tracing::error!("Internal server error: {}", error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::new(StatusCode::INTERNAL_SERVER_ERROR, error),
                )
            }
            Self::BadRequest(error) => {
                tracing::warn!("Bad request: {}", error);
                (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::new(StatusCode::BAD_REQUEST, error),
                )
            }
            Self::Unauthorized(error) => {
                tracing::warn!("Unauthorized access: {}", error);
                (
                    StatusCode::UNAUTHORIZED,
                    ErrorResponse::new(StatusCode::UNAUTHORIZED, error),
                )
            }
            Self::JsonRejection(rejection) => {
                tracing::error!("Bad user input: {:?}", rejection);
                (
                    rejection.status(),
                    ErrorResponse::new(rejection.status(), rejection.body_text()),
                )
            }
            Self::CustomError(status_code, message) => {
                tracing::error!("Error Custome code: {status_code} {message}");
                (status_code, ErrorResponse::new(status_code, message))
            }
            Self::PasswordHashError(error) => match error {
                argon2::password_hash::Error::Password => {
                    tracing::info!("Password mismatch error");
                    (
                        StatusCode::BAD_REQUEST,
                        ErrorResponse::new(
                            StatusCode::BAD_REQUEST,
                            "Email and Password combination does not match.".to_string(),
                        ),
                    )
                }
                _ => {
                    tracing::error!("Password hashing error: {}", error);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorResponse::new(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "An error occurred during password processing.".to_string(),
                        ),
                    )
                }
            },
            Self::UlidError(error) => {
                tracing::error!("UUID error: {}", error);
                (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::new(
                        StatusCode::BAD_REQUEST,
                        "Invalid UUID provided.".to_string(),
                    ),
                )
            }
            _ => {
                tracing::warn!("error: {}", self);
                (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::new(StatusCode::BAD_REQUEST, self.to_string()),
                )
            }
        };
        (status, Json(message)).into_response()
    }
}
