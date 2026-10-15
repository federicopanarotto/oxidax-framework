use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::response::Response;

use crate::api::users::user_dto::UserListQueryDto;
use crate::api::users::user_response::UserResponse;
use crate::api::users::user_service::UserService;
use crate::errors::AppResult;
use crate::framework::{CurrentUser, PaginationQuery, ValidatedQuery, route};

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
        (status = 401, description = "Missing or invalid access token"),
        (status = 404, description = "User not found"),
        (status = 503, description = "Database unavailable"),
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
        ("page" = Option<u64>, Query, description = "Page number, 1-based (default 1)", example = 1),
        ("limit" = Option<u64>, Query, description = "Elements per page, 1-100 (default 20)", example = 20),
    ),
    responses(
        (status = 200, description = "Paginated list of users (admin only); the total number of elements is returned in the `X-TOTAL-LIST` header", body = [UserResponse], headers(("X-TOTAL-LIST" = u64, description = "Total number of users across every page"))),
        (status = 401, description = "Missing or invalid access token"),
        (status = 403, description = "Role not allowed for this operation"),
        (status = 422, description = "Field validation failed"),
        (status = 503, description = "Database unavailable"),
    ),
    security(
        ("accessToken" = []),
    ),
)]
#[IsGranted(["admin"])]
pub async fn list_users(
    State(service): State<Arc<UserService>>,
    ValidatedQuery(query): ValidatedQuery<UserListQueryDto>,
    ValidatedQuery(pagination): ValidatedQuery<PaginationQuery>,
) -> AppResult<Response> {
    let page = service
        .list_page(query.role.as_deref(), pagination.pagination())
        .await?;
    Ok(page.into_response())
}
