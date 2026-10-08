//! Pipeline that turns errors into HTTP responses.
//!
//! Step 1: [`respond`] maps the error to its HTTP status, logs it and
//! serializes it into the uniform JSON body.
//! Step 2: the [`route_not_found`] fallback uses the same pipeline for unknown
//! routes.

use axum::Json;
use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use tracing::{error, warn};

use super::AppError;
use super::response::ErrorResponse;

/// Converts an [`AppError`] into the final HTTP response.
pub fn respond(error: AppError) -> Response {
    let status = error.status();
    let body = ErrorResponse {
        code: error.code().to_string(),
        error: status.to_string(),
        message: error.to_string(),
    };

    if status.is_server_error() {
        error!(
            code = %body.code,
            status = %status,
            message = %body.message,
            "internal error"
        );
    } else {
        warn!(
            code = %body.code,
            status = %status,
            message = %body.message,
            "request rejected"
        );
    }

    (status, Json(body)).into_response()
}

/// Fallback handler: any unknown route produces the same error shape as the
/// other endpoints.
pub async fn route_not_found(uri: Uri) -> Response {
    respond(AppError::NotFound(format!(
        "route {} not found",
        uri.path()
    )))
}
