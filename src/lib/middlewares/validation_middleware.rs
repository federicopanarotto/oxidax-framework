//! Validation extractors (JSON body, query string and path parameters).
//!
//! Each extractor deserializes the DTO and validates it with the
//! `#[validate(...)]` decorators of the `validator` crate, rejecting the
//! request before the controller is called. The error body is the same
//! [`ErrorResponse`] shape used by the rest of the API
//! (`{code, error, message}`):
//!
//! - malformed/unparsable body/query/params: `400 Bad Request`, code
//!   `bad_request`;
//! - invalid fields: `422 Unprocessable Entity`, code `validation_error`, with
//!   the per-field details joined into `message`.

use axum::Json;
use axum::extract::{FromRequest, FromRequestParts, Path, Query, Request};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::errors::ErrorResponse;
use crate::lib::validators::format_validation_errors;

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
                "bad_request",
                format!("Body JSON is invalid or malformed: {err}"),
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
                    "bad_request",
                    format!("Query parameters are invalid: {err}"),
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
                    "bad_request",
                    format!("Path parameters are invalid: {err}"),
                )
            })?;

        if let Err(errors) = data.validate() {
            return Err(validation_error_response("Params", errors));
        }

        Ok(Self(data))
    }
}

fn validation_error_response(source: &str, errors: validator::ValidationErrors) -> Response {
    let details = format_validation_errors(errors).join("; ");
    error_response(
        StatusCode::UNPROCESSABLE_ENTITY,
        "validation_error",
        format!("Validation error on {source}: {details}"),
    )
}

fn error_response(status: StatusCode, code: &str, message: impl Into<String>) -> Response {
    (status, Json(ErrorResponse::new(code, status, message))).into_response()
}
