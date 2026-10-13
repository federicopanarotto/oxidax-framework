//! Centralized error handling.
//!
//! Pipeline: every application error is normalized into [`AppError`] (machine
//! code + HTTP status), then [`pipeline::respond`] turns it into the uniform
//! JSON response with structured logging. DTO validation errors handled by
//! the extractors in `crate` share the same [`ErrorResponse`] body.

pub mod pipeline;
pub mod response;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use utoipa::openapi::RefOr;
use utoipa::openapi::schema::{Ref, Schema};
use utoipa::{PartialSchema, ToSchema};

pub use response::ErrorResponse;

/// OpenAPI alias of the `400 Bad Request` responses of the write methods:
/// whatever the source (malformed body/query from the validation extractors or
/// an application error such as a missing category), the body has the same
/// [`ErrorResponse`] shape. It exists only for documentation, not at runtime.
pub struct BadRequestResponse {}

impl PartialSchema for BadRequestResponse {
    fn schema() -> RefOr<Schema> {
        Ref::from_schema_name("ErrorResponse").into()
    }
}

impl ToSchema for BadRequestResponse {
    fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
        schemas.push((Self::name().into_owned(), Self::schema()));
    }
}

/// Generic application error, independent from the layer that produces it.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    /// Request body too large (upload limits exceeded).
    #[error("{0}")]
    PayloadTooLarge(String),
    /// Unsupported media type (e.g. a rejected file extension).
    #[error("{0}")]
    UnsupportedMediaType(String),
    #[error("{0}")]
    Database(String),
    #[error("{0}")]
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    /// Stable machine code, exposed in the response body.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "not_found",
            Self::Validation(_) => "validation_error",
            Self::Conflict(_) => "conflict",
            Self::Unauthorized(_) => "unauthorized",
            Self::Forbidden(_) => "forbidden",
            Self::PayloadTooLarge(_) => "payload_too_large",
            Self::UnsupportedMediaType(_) => "unsupported_media_type",
            Self::Database(_) => "database_unavailable",
            Self::Internal(_) => "internal_error",
        }
    }

    /// HTTP status associated with the error.
    pub fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            Self::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::Database(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        pipeline::respond(self)
    }
}

impl From<surrealdb::Error> for AppError {
    fn from(err: surrealdb::Error) -> Self {
        Self::Database(err.to_string())
    }
}
