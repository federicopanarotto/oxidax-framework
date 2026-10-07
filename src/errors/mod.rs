//! Gestione centralizzata degli errori.
//!
//! Pipeline: ogni errore dell'applicazione viene normalizzato in [`AppError`]
//! (codice macchina + stato HTTP), poi [`pipeline::respond`] lo converte nella
//! risposta JSON uniforme con log strutturato. Anche i rifiuti degli extractor
//! rientrano nella stessa pipeline tramite [`AppJson`].

pub mod extract;
pub mod pipeline;
pub mod response;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub use extract::AppJson;
pub use response::ErrorResponse;

/// Errore generico dell'applicazione, indipendente dal layer che lo produce.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Database(String),
    #[error("{0}")]
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    /// Codice macchina stabile, esposto nel corpo della risposta.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "not_found",
            Self::Validation(_) => "validation_error",
            Self::Database(_) => "database_unavailable",
            Self::Internal(_) => "internal_error",
        }
    }

    /// Stato HTTP associato all'errore.
    pub fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
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
