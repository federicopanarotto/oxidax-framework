use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::api::categories::category_dto::{
    CategoryResponse, CreateCategoryDto, UpdateCategoryDto,
};
use crate::api::categories::category_service::CategoryService;
use crate::errors::{AppJson, AppResult, ErrorResponse};

/// Controller delle categorie: incapsula il service ed espone gli endpoint.
///
/// Come per i report, i metodi contengono la logica HTTP (status code + delega
/// al service); le funzioni con `#[utoipa::path]` sono i handler axum, sottili
/// wrapper che estraggono la richiesta e documentano l'endpoint in OpenAPI.
#[derive(Clone)]
pub struct CategoryController {
    service: CategoryService,
}

impl CategoryController {
    pub fn new(service: CategoryService) -> Self {
        Self { service }
    }

    /// Crea una categoria: 201 con la categoria appena inserita.
    pub async fn create(
        &self,
        dto: CreateCategoryDto,
    ) -> AppResult<(StatusCode, CategoryResponse)> {
        let created = self.service.create(dto).await?;
        Ok((StatusCode::CREATED, created))
    }

    /// Elenca tutte le categorie.
    pub async fn list(&self) -> AppResult<Vec<CategoryResponse>> {
        self.service.list().await
    }

    /// Recupera una categoria per id.
    pub async fn get(&self, id: &str) -> AppResult<CategoryResponse> {
        self.service.get(id).await
    }

    /// Sostituisce una categoria esistente.
    pub async fn update(&self, id: &str, dto: UpdateCategoryDto) -> AppResult<CategoryResponse> {
        self.service.update(id, dto).await
    }

    /// Elimina una categoria: 204 senza corpo.
    pub async fn delete(&self, id: &str) -> AppResult<StatusCode> {
        self.service.delete(id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}

#[utoipa::path(
    post,
    path = "/api/categories",
    tag = "categories",
    request_body = CreateCategoryDto,
    responses(
        (status = 201, description = "Category created", body = CategoryResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn create_category(
    State(controller): State<CategoryController>,
    AppJson(dto): AppJson<CreateCategoryDto>,
) -> AppResult<(StatusCode, Json<CategoryResponse>)> {
    let (status, created) = controller.create(dto).await?;
    Ok((status, Json(created)))
}

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
    State(controller): State<CategoryController>,
) -> AppResult<Json<Vec<CategoryResponse>>> {
    Ok(Json(controller.list().await?))
}

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
    State(controller): State<CategoryController>,
    Path(id): Path<String>,
) -> AppResult<Json<CategoryResponse>> {
    Ok(Json(controller.get(&id).await?))
}

#[utoipa::path(
    put,
    path = "/api/categories/{id}",
    tag = "categories",
    params(("id" = String, Path, description = "Category id")),
    request_body = UpdateCategoryDto,
    responses(
        (status = 200, description = "Category updated", body = CategoryResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 404, description = "Category not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn update_category(
    State(controller): State<CategoryController>,
    Path(id): Path<String>,
    AppJson(dto): AppJson<UpdateCategoryDto>,
) -> AppResult<Json<CategoryResponse>> {
    Ok(Json(controller.update(&id, dto).await?))
}

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
    State(controller): State<CategoryController>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    controller.delete(&id).await
}
