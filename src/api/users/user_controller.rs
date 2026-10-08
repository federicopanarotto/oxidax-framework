use std::sync::Arc;

use axum::Json;
use axum::extract::State;

use crate::api::users::user_dto::{UserListQueryDto, UserResponse};
use crate::api::users::user_service::UserService;
use crate::errors::{AppResult, ErrorResponse};
use crate::framework::{CurrentUser, ValidatedQuery, ValidationErrorResponse, route};

// Route table and OpenAPI spec of the users, defined point-inline: every
// handler is annotated with `#[utoipa::path(...)]`, `#[IsGranted]`
// (consumed by `#[route]`: it installs the guard and gives access to the
// `CurrentUser` extractor) and `#[route]`, which self-registers it in the
// static registry. `features.rs`/`router.rs`/`docs.rs` never change when a
// handler is added.

#[route]
#[utoipa::path(
    get,
    path = "/api/users/me",
    tag = "users",
    responses(
        (status = 200, description = "Profile of the authenticated user", body = UserResponse),
        (status = 401, description = "Missing or invalid access token", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
    security(
        ("accessToken" = []),
    ),
)]
#[IsGranted]
pub async fn get_me(
    State(service): State<Arc<UserService>>,
    current: CurrentUser,
) -> AppResult<Json<UserResponse>> {
    Ok(Json(service.me(&current.user_id).await?))
}

#[route]
#[utoipa::path(
    get,
    path = "/api/users",
    tag = "users",
    params(
        ("role" = Option<String>, Query, description = "Role filter: `admin` or `user`; absent returns every user", example = "admin"),
    ),
    responses(
        (status = 200, description = "List of users (admin only)", body = [UserResponse]),
        (status = 401, description = "Missing or invalid access token", body = ErrorResponse),
        (status = 403, description = "Role not allowed for this operation", body = ErrorResponse),
        (status = 422, description = "Field validation failed", body = ValidationErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
    security(
        ("accessToken" = []),
    ),
)]
#[IsGranted(["admin"])]
pub async fn list_users(
    State(service): State<Arc<UserService>>,
    ValidatedQuery(query): ValidatedQuery<UserListQueryDto>,
) -> AppResult<Json<Vec<UserResponse>>> {
    Ok(Json(service.list(query.role.as_deref()).await?))
}
