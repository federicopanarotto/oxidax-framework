use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use utoipa::openapi::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::categories::category_dto::{
    CategoryResponse, CreateCategoryDto, UpdateCategoryDto,
};
use crate::api::categories::category_service::CategoryService;
use crate::errors::{AppResult, BadRequestResponse, ErrorResponse};
use crate::lib::{ValidatedJson, ValidationErrorResponse};

/// Route table (axum) and OpenAPI spec for categories, defined right here,
/// without an object controller: the handlers are `#[utoipa::path]`
/// functions that use [`CategoryService`] as state.
///
/// Registration in the DI container and in the feature registry (`order` 0)
/// live in `mod.rs`, so `features.rs`/`router.rs`/`docs.rs` never change.
pub fn build_router(service: Arc<CategoryService>) -> (Router, OpenApi) {
    OpenApiRouter::new()
        .routes(routes!(create_category, list_categories))
        .routes(routes!(get_category, update_category, delete_category))
        .with_state(service)
        .split_for_parts()
}

#[utoipa::path(
    post,
    path = "/api/categories",
    tag = "categories",
    request_body = CreateCategoryDto,
    responses(
        (status = 201, description = "Categoria creata con successo", body = CategoryResponse),
        (status = 400, description = "Richiesta non valida: body JSON malformato", body = BadRequestResponse),
        (status = 422, description = "Validazione dei campi non superata", body = ValidationErrorResponse),
        (status = 503, description = "Database non disponibile", body = ErrorResponse),
    ),
)]
pub async fn create_category(
    State(service): State<Arc<CategoryService>>,
    ValidatedJson(dto): ValidatedJson<CreateCategoryDto>,
) -> AppResult<(StatusCode, Json<CategoryResponse>)> {
    let created = service.create(dto).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    get,
    path = "/api/categories",
    tag = "categories",
    responses(
        (status = 200, description = "Elenco delle categorie", body = [CategoryResponse]),
        (status = 503, description = "Database non disponibile", body = ErrorResponse),
    ),
)]
pub async fn list_categories(
    State(service): State<Arc<CategoryService>>,
) -> AppResult<Json<Vec<CategoryResponse>>> {
    Ok(Json(service.list().await?))
}

#[utoipa::path(
    get,
    path = "/api/categories/{id}",
    tag = "categories",
    params(("id" = String, Path, description = "Id della categoria")),
    responses(
        (status = 200, description = "Categoria trovata", body = CategoryResponse),
        (status = 404, description = "Categoria non trovata", body = ErrorResponse),
        (status = 503, description = "Database non disponibile", body = ErrorResponse),
    ),
)]
pub async fn get_category(
    State(service): State<Arc<CategoryService>>,
    Path(id): Path<String>,
) -> AppResult<Json<CategoryResponse>> {
    Ok(Json(service.get(&id).await?))
}

#[utoipa::path(
    put,
    path = "/api/categories/{id}",
    tag = "categories",
    params(("id" = String, Path, description = "Id della categoria")),
    request_body = UpdateCategoryDto,
    responses(
        (status = 200, description = "Categoria aggiornata", body = CategoryResponse),
        (status = 400, description = "Richiesta non valida: body JSON malformato", body = BadRequestResponse),
        (status = 404, description = "Categoria non trovata", body = ErrorResponse),
        (status = 422, description = "Validazione dei campi non superata", body = ValidationErrorResponse),
        (status = 503, description = "Database non disponibile", body = ErrorResponse),
    ),
)]
pub async fn update_category(
    State(service): State<Arc<CategoryService>>,
    Path(id): Path<String>,
    ValidatedJson(dto): ValidatedJson<UpdateCategoryDto>,
) -> AppResult<Json<CategoryResponse>> {
    Ok(Json(service.update(&id, dto).await?))
}

#[utoipa::path(
    delete,
    path = "/api/categories/{id}",
    tag = "categories",
    params(("id" = String, Path, description = "Id della categoria")),
    responses(
        (status = 204, description = "Categoria eliminata"),
        (status = 404, description = "Categoria non trovata", body = ErrorResponse),
        (status = 503, description = "Database non disponibile", body = ErrorResponse),
    ),
)]
pub async fn delete_category(
    State(service): State<Arc<CategoryService>>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    service.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
