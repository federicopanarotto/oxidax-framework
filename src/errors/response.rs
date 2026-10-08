use serde::Serialize;
use utoipa::ToSchema;

/// Uniform JSON body of every API error.
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
