use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Serialize;

use crate::{responses::Json, Result};

/// # Errors
///
/// Currently this function did't return any error. this is for feature
/// functionality
pub fn text(c: &str) -> Result<Response> {
    Ok(c.to_string().into_response())
}

/// # Errors
///
/// Currently this function did't return any error. this is for feature
/// functionality
pub fn json<T: Serialize>(c: T) -> Result<Response> {
    Ok(Json(c).into_response())
}

/// # Errors
///
/// Currently this function did't return any error. this is for feature
/// functionality
pub fn html(content: &str) -> Result<Response> {
    Ok(Html(content.to_string()).into_response())
}

/// # Errors
///
/// Currently this function did't return any error. this is for feature
/// functionality
pub fn redirect(to: &str) -> Result<Response> {
    Ok(Redirect::to(to).into_response())
}
