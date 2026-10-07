//! Gestione centralizzata degli errori.
//!
//! Pipeline: ogni errore dell'applicazione viene normalizzato in [`AppError`]
//! (codice macchina + stato HTTP), poi [`pipeline::respond`] lo converte nella
//! risposta JSON uniforme con log strutturato. Gli errori di validazione dei
//! DTO vengono gestiti invece dagli estrattori in `crate::lib` con il formato
//! `{error, details}`.

pub mod pipeline;
pub mod response;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use utoipa::openapi::RefOr;
use utoipa::openapi::schema::{OneOfBuilder, Ref, Schema};
use utoipa::{PartialSchema, ToSchema};

pub use response::ErrorResponse;

/// Corpo delle risposte `400 Bad Request` dei metodi di scrittura: può essere
/// il rifiuto degli estrattori di validazione per body/query mal formati
/// ([`ValidationErrorResponse`], shape `{error, details}`) oppure un errore di
/// dominio come la categoria inesistente ([`ErrorResponse`], shape
/// `{code, error, message}`).
///
/// È una semplice alias OpenAPI: non esiste a runtime, serve solo a
/// documentare che il 400 può restituire uno dei due corpi (schema `oneOf`).
pub struct BadRequestResponse {}

impl PartialSchema for BadRequestResponse {
    fn schema() -> RefOr<Schema> {
        OneOfBuilder::new()
            .item(Ref::from_schema_name("ErrorResponse"))
            .item(Ref::from_schema_name("ValidationErrorResponse"))
            .into()
    }
}

impl ToSchema for BadRequestResponse {
    fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
        schemas.push((Self::name().into_owned(), Self::schema()));
    }
}

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
