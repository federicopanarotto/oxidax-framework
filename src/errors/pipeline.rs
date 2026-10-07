//! Pipeline di conversione degli errori in risposte HTTP.
//!
//! Fase 1: [`respond`] mappa l'errore sullo stato HTTP, lo logga e lo
//! serializza nel corpo JSON uniforme.
//! Fase 2: il fallback [`route_not_found`] usa la stessa pipeline per le
//! rotte inesistenti.

use axum::Json;
use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use tracing::{error, warn};

use super::AppError;
use super::response::ErrorResponse;

/// Converte un [`AppError`] nella risposta HTTP finale.
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
            "errore interno"
        );
    } else {
        warn!(
            code = %body.code,
            status = %status,
            message = %body.message,
            "richiesta rifiutata"
        );
    }

    (status, Json(body)).into_response()
}

/// Handler di fallback: qualsiasi rotta sconosciuta produce lo stesso
/// formato di errore degli altri endpoint.
pub async fn route_not_found(uri: Uri) -> Response {
    respond(AppError::NotFound(format!(
        "route {} not found",
        uri.path()
    )))
}
