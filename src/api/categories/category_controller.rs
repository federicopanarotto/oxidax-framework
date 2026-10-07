use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;

use crate::api::categories::category_dto::{CategoryResponse, CreateCategoryDto, UpdateCategoryDto};
use crate::api::categories::category_service::CategoryService;
use crate::errors::{AppResult, BadRequestResponse, ErrorResponse};
use crate::lib::{ValidatedJson, ValidationErrorResponse};

/// Tabella delle rotte delle categorie, costruita sul service iniettato come
/// stato.
///
/// Come per i report gli handler sono funzioni `#[utoipa::path]` che usano
/// direttamente il [`CategoryService`]: niente controller oggetto; l'extractor
/// [`ValidatedJson`] applica i decoratori `#[validate(...)]` del DTO prima
/// dell'esecuzione.
pub fn router(service: Arc<CategoryService>) -> Router {
    Router::new()
        .route("/categories", post(create_category).get(list_categories))
        .route(
            "/categories/{id}",
            get(get_category)
                .put(update_category)
                .delete(delete_category),
        )
        .with_state(service)
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