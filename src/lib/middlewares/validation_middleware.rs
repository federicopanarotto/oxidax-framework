//! Estrattori di validazione (body JSON, query string e parametri di path).
//!
//! Ogni estrattore deserializza il DTO e ne esegue la validazione con i
//! decoratori `#[validate(...)]` del crate `validator`, rifiutando la
//! richiesta con un corpo JSON uniforme prima che il controller venga chiamato:
//!
//! - JSON malformato/parsabile: `400 Bad Request`;
//! - campi non validi: `422 Unprocessable Entity` con i dettagli per campo.

use axum::Json;
use axum::extract::{FromRequest, FromRequestParts, Path, Query, Request};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde::de::DeserializeOwned;
use utoipa::ToSchema;
use validator::Validate;

use crate::lib::validators::format_validation_errors;

/// Corpo JSON uniforme degli errori di validazione degli estrattori.
#[derive(Debug, Serialize, ToSchema)]
pub struct ValidationErrorResponse {
    /// Descrizione dell'errore (body/query/params)
    #[schema(example = "Errore di validazione del Body")]
    pub error: String,
    /// Dettagli per ogni campo (`campo: messaggio`)
    pub details: Vec<String>,
}

/// DTO validato proveniente dal body JSON.
pub struct ValidatedJson<T>(pub T);

/// DTO validato proveniente dalla query string (`?page=1&search=abc`).
pub struct ValidatedQuery<T>(pub T);

/// DTO validato proveniente dai parametri di path (`/api/reports/{id}`).
pub struct ValidatedParams<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Send + Validate,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(data) = Json::<T>::from_request(req, state)
            .await
            .map_err(|err| {
                error_response(
                    StatusCode::BAD_REQUEST,
                    "Body JSON non valido o malformato",
                    vec![err.to_string()],
                )
            })?;

        if let Err(errors) = data.validate() {
            return Err(validation_error_response("Body", errors));
        }

        Ok(Self(data))
    }
}

impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Send + Validate,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(data) = Query::<T>::from_request_parts(parts, state)
            .await
            .map_err(|err| {
                error_response(
                    StatusCode::BAD_REQUEST,
                    "Query Parameters non validi",
                    vec![err.to_string()],
                )
            })?;

        if let Err(errors) = data.validate() {
            return Err(validation_error_response("Query", errors));
        }

        Ok(Self(data))
    }
}

impl<T, S> FromRequestParts<S> for ValidatedParams<T>
where
    T: DeserializeOwned + Send + Validate,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(data) = Path::<T>::from_request_parts(parts, state)
            .await
            .map_err(|err| {
                error_response(
                    StatusCode::BAD_REQUEST,
                    "Parametri di path non validi",
                    vec![err.to_string()],
                )
            })?;

        if let Err(errors) = data.validate() {
            return Err(validation_error_response("Params", errors));
        }

        Ok(Self(data))
    }
}

fn validation_error_response(source: &str, errors: validator::ValidationErrors) -> Response {
    error_response(
        StatusCode::UNPROCESSABLE_ENTITY,
        format!("Errore di validazione del {source}"),
        format_validation_errors(errors),
    )
}

fn error_response(status: StatusCode, error: impl Into<String>, details: Vec<String>) -> Response {
    (status, Json(ValidationErrorResponse { error: error.into(), details })).into_response()
}