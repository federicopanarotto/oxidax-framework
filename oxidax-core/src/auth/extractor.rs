//! Extractor of the authenticated caller.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};

use crate::errors::AppError;

/// Authenticated caller inserted in the request extensions by
/// [`crate::auth::guard::guard`].
///
/// Use it on a handler annotated with `#[route]` + `#[IsGranted]` (the guard
/// that fills it is installed by `#[IsGranted]`): without it the extractor
/// answers `401`.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    /// Id of the user.
    pub user_id: String,
    /// Id of the [`crate::auth::UserIdentity`] used to authenticate.
    pub identity_id: String,
    /// Role of the user at the time the access token was issued.
    pub role: String,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or_else(|| {
                AppError::Unauthorized("authentication required".to_string()).into_response()
            })
    }
}
