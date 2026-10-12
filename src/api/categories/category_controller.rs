use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;

use crate::api::categories::category_dto::{
    CategoryResponse, CreateCategoryDto, UpdateCategoryDto,
};
use crate::api::categories::category_service::CategoryService;
use crate::errors::{AppResult, BadRequestResponse, ErrorResponse};
use crate::framework::{PaginationQuery, ValidatedJson, ValidatedQuery, route};

// Route table and OpenAPI spec of the categories, defined point-inline: every
// handler is annotated with `#[utoipa::path(...)]` (HTTP method, path and
// responses) and `#[route]`, which self-registers it in the static registry.
// `features.rs`/`router.rs`/`docs.rs` never change when a handler is added.

#[route]
#[utoipa::path(
    get,
    path = "/api/categories",
    tag = "categories",
    params(
        ("page" = Option<u64>, Query, description = "Page number, 1-based (default 1)", example = 1),
        ("limit" = Option<u64>, Query, description = "Elements per page, 1-100 (default 20)", example = 20),
    ),
    responses(
        (status = 200, description = "Paginated list of categories; the total number of elements is returned in the `X-TOTAL-LIST` header", body = [CategoryResponse], headers(("X-TOTAL-LIST" = u64, description = "Total number of categories across every page"))),
        (status = 422, description = "Invalid pagination parameters", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn list_categories(
    State(service): State<Arc<CategoryService>>,
    ValidatedQuery(pagination): ValidatedQuery<PaginationQuery>,
) -> AppResult<Response> {
    let page = service.list_page(pagination.pagination()).await?;
    Ok(page.into_response())
}

#[route]
#[utoipa::path(
    post,
    path = "/api/categories",
    tag = "categories",
    request_body = CreateCategoryDto,
    responses(
        (status = 201, description = "Category created successfully", body = CategoryResponse),
        (status = 400, description = "Invalid request: malformed JSON body", body = BadRequestResponse),
        (status = 422, description = "Field validation failed", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn create_category(
    State(service): State<Arc<CategoryService>>,
    ValidatedJson(dto): ValidatedJson<CreateCategoryDto>,
) -> AppResult<(StatusCode, Json<CategoryResponse>)> {
    let created = service.create(dto).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[route]
#[utoipa::path(
    get,
    path = "/api/categories/{id}",
    tag = "categories",
    params(("id" = String, Path, description = "Category id")),
    responses(
        (status = 200, description = "Category found", body = CategoryResponse),
        (status = 404, description = "Category not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn get_category(
    State(service): State<Arc<CategoryService>>,
    Path(id): Path<String>,
) -> AppResult<Json<CategoryResponse>> {
    Ok(Json(service.get(&id).await?))
}

#[route]
#[utoipa::path(
    put,
    path = "/api/categories/{id}",
    tag = "categories",
    params(("id" = String, Path, description = "Category id")),
    request_body = UpdateCategoryDto,
    responses(
        (status = 200, description = "Category updated", body = CategoryResponse),
        (status = 400, description = "Invalid request: malformed JSON body", body = BadRequestResponse),
        (status = 404, description = "Category not found", body = ErrorResponse),
        (status = 422, description = "Field validation failed", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn update_category(
    State(service): State<Arc<CategoryService>>,
    Path(id): Path<String>,
    ValidatedJson(dto): ValidatedJson<UpdateCategoryDto>,
) -> AppResult<Json<CategoryResponse>> {
    Ok(Json(service.update(&id, dto).await?))
}

#[route]
#[utoipa::path(
    delete,
    path = "/api/categories/{id}",
    tag = "categories",
    params(("id" = String, Path, description = "Category id")),
    responses(
        (status = 204, description = "Category deleted"),
        (status = 404, description = "Category not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn delete_category(
    State(service): State<Arc<CategoryService>>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    service.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
