use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::api::places::place_dto::{CreatePlaceDto, PlaceResponse, UpdatePlaceDto};
use crate::api::places::place_service::PlaceService;
use crate::errors::{AppResult, BadRequestResponse, ErrorResponse};
use crate::framework::{ValidatedJson, ValidationErrorResponse, route};

// Route table and OpenAPI spec of the places, defined point-inline: every
// handler is annotated with `#[utoipa::path(...)]` (HTTP method, path and
// responses) and `#[route]`, which self-registers it in the static registry.
// `features.rs`/`router.rs`/`docs.rs` never change when a handler is added.

#[route]
#[utoipa::path(
    post,
    path = "/api/places",
    tag = "places",
    request_body = CreatePlaceDto,
    responses(
        (status = 201, description = "Place created successfully", body = PlaceResponse),
        (status = 400, description = "Invalid request: malformed JSON body", body = BadRequestResponse),
        (status = 422, description = "Field validation failed", body = ValidationErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn create_place(
    State(service): State<Arc<PlaceService>>,
    ValidatedJson(dto): ValidatedJson<CreatePlaceDto>,
) -> AppResult<(StatusCode, Json<PlaceResponse>)> {
    let created = service.create(dto).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[route]
#[utoipa::path(
    get,
    path = "/api/places",
    tag = "places",
    responses(
        (status = 200, description = "List of places", body = [PlaceResponse]),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn list_places(
    State(service): State<Arc<PlaceService>>,
) -> AppResult<Json<Vec<PlaceResponse>>> {
    Ok(Json(service.list().await?))
}

#[route]
#[utoipa::path(
    get,
    path = "/api/places/{id}",
    tag = "places",
    params(("id" = String, Path, description = "Place id")),
    responses(
        (status = 200, description = "Place found", body = PlaceResponse),
        (status = 404, description = "Place not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn get_place(
    State(service): State<Arc<PlaceService>>,
    Path(id): Path<String>,
) -> AppResult<Json<PlaceResponse>> {
    Ok(Json(service.get(&id).await?))
}

#[route]
#[utoipa::path(
    put,
    path = "/api/places/{id}",
    tag = "places",
    params(("id" = String, Path, description = "Place id")),
    request_body = UpdatePlaceDto,
    responses(
        (status = 200, description = "Place updated", body = PlaceResponse),
        (status = 400, description = "Invalid request: malformed JSON body", body = BadRequestResponse),
        (status = 404, description = "Place not found", body = ErrorResponse),
        (status = 422, description = "Field validation failed", body = ValidationErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn update_place(
    State(service): State<Arc<PlaceService>>,
    Path(id): Path<String>,
    ValidatedJson(dto): ValidatedJson<UpdatePlaceDto>,
) -> AppResult<Json<PlaceResponse>> {
    Ok(Json(service.update(&id, dto).await?))
}

#[route]
#[utoipa::path(
    delete,
    path = "/api/places/{id}",
    tag = "places",
    params(("id" = String, Path, description = "Place id")),
    responses(
        (status = 204, description = "Place deleted"),
        (status = 404, description = "Place not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn delete_place(
    State(service): State<Arc<PlaceService>>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    service.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
