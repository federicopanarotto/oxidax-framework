use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::api::categories::category_dto::{
    CategoryResponse, CreateCategoryDto, UpdateCategoryDto,
};
use crate::api::categories::category_service::CategoryService;
use crate::errors::{AppResult, BadRequestResponse, ErrorResponse};
use crate::framework::{ValidatedJson, ValidationErrorResponse, route};

// Route table and OpenAPI spec of the categories, defined point-inline: every
// handler is annotated with `#[utoipa::path(...)]` (HTTP method, path and
// responses) and `#[route]`, which self-registers it in the static registry.
// `features.rs`/`router.rs`/`docs.rs` never change when a handler is added.

#[route]
#[utoipa::path(
    post,
    path = "/api/categories",
    tag = "categories",
    request_body = CreateCategoryDto,
    responses(
        (status = 201, description = "Category created successfully", body = CategoryResponse),
        (status = 400, description = "Invalid request: malformed JSON body", body = BadRequestResponse),
        (status = 422, description = "Field validation failed", body = ValidationErrorResponse),
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
    path = "/api/categories",
    tag = "categories",
    responses(
        (status = 200, description = "List of categories", body = [CategoryResponse]),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn list_categories(
    State(service): State<Arc<CategoryService>>,
) -> AppResult<Json<Vec<CategoryResponse>>> {
    Ok(Json(service.list().await?))
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
        (status = 422, description = "Field validation failed", body = ValidationErrorResponse),
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
