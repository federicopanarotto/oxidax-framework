use serde::Serialize;
use utoipa::ToSchema;

/// Corpo JSON uniforme di tutti gli errori dell'API.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Codice macchina dell'errore
    #[schema(example = "not_found")]
    pub code: String,
    /// Stato HTTP
    #[schema(example = "404 Not Found")]
    pub error: String,
    /// Messaggio leggibile
    #[schema(example = "report 550e8400... not found")]
    pub message: String,
}
