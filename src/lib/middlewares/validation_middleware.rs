//! Validation extractors (JSON body, query string and path parameters).
//!
//! Each extractor deserializes the DTO and validates it with the
//! `#[validate(...)]` decorators of the `validator` crate, rejecting the
//! request with a uniform JSON body before the controller is called:
//!
//! - malformed/unparsable JSON: `400 Bad Request`;
//! - invalid fields: `422 Unprocessable Entity` with per-field details.

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

/// Uniform JSON body of the extractors' validation errors.
#[derive(Debug, Serialize, ToSchema)]
pub struct ValidationErrorResponse {
    /// Description of the error (body/query/params)
    #[schema(example = "Validation error on body")]
    pub error: String,
    /// Details for each field (`field: message`)
    pub details: Vec<String>,
}

/// Validated DTO coming from the JSON body.
pub struct ValidatedJson<T>(pub T);

/// Validated DTO coming from the query string (`?page=1&search=abc`).
pub struct ValidatedQuery<T>(pub T);

/// Validated DTO coming from the path parameters (`/api/reports/{id}`).
pub struct ValidatedParams<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Send + Validate,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(data) = Json::<T>::from_request(req, state).await.map_err(|err| {
            error_response(
                StatusCode::BAD_REQUEST,
                "Body JSON is invalid or malformed",
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
                    "Query parameters are invalid",
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
                    "Path parameters are invalid",
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
        format!("Validation error on {source}"),
        format_validation_errors(errors),
    )
}

fn error_response(status: StatusCode, error: impl Into<String>, details: Vec<String>) -> Response {
    (
        status,
        Json(ValidationErrorResponse {
            error: error.into(),
            details,
        }),
    )
        .into_response()
}
