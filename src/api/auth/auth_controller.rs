use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::api::auth::auth_dto::{
    LoginRequest, MessageResponse, RefreshRequest, RegisterRequest, TokenResponse,
};
use crate::api::auth::auth_service::AuthService;
use crate::errors::{AppResult, BadRequestResponse, ErrorResponse};
use crate::framework::{CurrentUser, ValidatedJson, ValidationErrorResponse, route};

// Route table and OpenAPI spec of the authentication, defined point-inline:
// every handler is annotated with `#[utoipa::path(...)]`, `#[IsGranted]`
// (only on `logout`: it installs the guard and gives access to the
// `CurrentUser` extractor) and `#[route]`, which self-registers it in the
// static registry. `features.rs`/`router.rs`/`docs.rs` never change when a
// handler is added.

#[route]
#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = MessageResponse),
        (status = 400, description = "Invalid request: malformed JSON body", body = BadRequestResponse),
        (status = 409, description = "Registration could not be completed", body = ErrorResponse),
        (status = 422, description = "Field validation failed", body = ValidationErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn register(
    State(service): State<Arc<AuthService>>,
    ValidatedJson(dto): ValidatedJson<RegisterRequest>,
) -> AppResult<(StatusCode, Json<MessageResponse>)> {
    let created = service.register(dto).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[route]
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Tokens issued", body = TokenResponse),
        (status = 400, description = "Invalid request: malformed JSON body", body = BadRequestResponse),
        (status = 401, description = "Invalid credentials (wrong email or wrong password)", body = ErrorResponse),
        (status = 422, description = "Field validation failed", body = ValidationErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn login(
    State(service): State<Arc<AuthService>>,
    ValidatedJson(dto): ValidatedJson<LoginRequest>,
) -> AppResult<Json<TokenResponse>> {
    Ok(Json(service.login(dto).await?))
}

#[route]
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "New token pair issued; the previous refresh token is no longer valid", body = TokenResponse),
        (status = 400, description = "Invalid request: malformed JSON body", body = BadRequestResponse),
        (status = 401, description = "Invalid, expired or already used refresh token", body = ErrorResponse),
        (status = 422, description = "Field validation failed", body = ValidationErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
    security(
        ("refreshToken" = []),
    ),
)]
pub async fn refresh_session(
    State(service): State<Arc<AuthService>>,
    ValidatedJson(dto): ValidatedJson<RefreshRequest>,
) -> AppResult<Json<TokenResponse>> {
    Ok(Json(service.refresh(dto).await?))
}

#[route]
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "auth",
    responses(
        (status = 204, description = "Session revoked: the outstanding refresh token no longer works"),
        (status = 401, description = "Missing or invalid access token", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
    security(
        ("accessToken" = []),
    ),
)]
#[IsGranted]
pub async fn logout(
    State(service): State<Arc<AuthService>>,
    current: CurrentUser,
) -> AppResult<StatusCode> {
    service.logout(&current).await?;
    Ok(StatusCode::NO_CONTENT)
}
