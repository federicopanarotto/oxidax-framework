//! Authentication and role guard.
//!
//! Installed as a route layer by `#[IsGranted]` (the attribute consumed by
//! `#[route]`): it validates the `Authorization: Bearer` header, answers
//! `401` when the token is missing/invalid and `403` when the role of the
//! caller is not among the allowed ones, then publishes the
//! [`CurrentUser`] in the request extensions for the handler's extractor.

use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::errors::AppError;
use crate::lib::auth::extractor::CurrentUser;
use crate::lib::auth::jwt::JwtConfig;

/// Guard state injected by the generated route builder: the JWT settings and
/// the roles allowed to call the endpoint (an empty list means "any
/// authenticated caller").
pub type GuardState = (JwtConfig, Vec<String>);

/// Validates the access token and the role of the caller.
pub async fn guard(
    State((jwt, roles)): State<GuardState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(token) = bearer_token(&request) else {
        return AppError::Unauthorized("authentication required".to_string()).into_response();
    };

    let claims = match jwt.verify_access(&token) {
        Ok(claims) => claims,
        Err(_) => {
            return AppError::Unauthorized("invalid or expired access token".to_string())
                .into_response();
        }
    };

    if !roles.is_empty() && !roles.iter().any(|role| role == claims.role.as_str()) {
        return AppError::Forbidden("insufficient role".to_string()).into_response();
    }

    request.extensions_mut().insert(CurrentUser {
        user_id: claims.sub,
        identity_id: claims.identity_id,
        role: claims.role,
    });

    next.run(request).await
}

/// Extracts the token from `Authorization: Bearer <token>`.
fn bearer_token(request: &Request) -> Option<String> {
    let value = request
        .headers()
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    let (scheme, token) = value.split_once(' ')?;
    let token = token.trim();
    if scheme.eq_ignore_ascii_case("bearer") && !token.is_empty() {
        Some(token.to_string())
    } else {
        None
    }
}
