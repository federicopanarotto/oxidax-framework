use axum::http::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

/// Uniform JSON body of every API error, including DTO validation failures.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Machine code of the error
    #[schema(example = "not_found")]
    pub code: String,
    /// HTTP status
    #[schema(example = "404 Not Found")]
    pub error: String,
    /// Human-readable message
    #[schema(example = "report 550e8400... not found")]
    pub message: String,
}

impl ErrorResponse {
    /// Builds the uniform body from a stable `code`, an HTTP `status` and a
    /// human-readable `message`. Used both by the error pipeline and by the
    /// validation extractors, so every error shares one shape.
    pub fn new(code: impl Into<String>, status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            error: status.to_string(),
            message: message.into(),
        }
    }
}
