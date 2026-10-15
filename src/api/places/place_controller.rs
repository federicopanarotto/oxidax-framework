use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;

use crate::api::places::place_dto::{CreatePlaceDto, UpdatePlaceDto};
use crate::api::places::place_response::PlaceResponse;
use crate::api::places::place_service::PlaceService;
use crate::errors::AppResult;
use crate::framework::{PaginationQuery, ValidatedJson, ValidatedQuery, route};

// Route table and OpenAPI spec of the places, defined point-inline: every
// handler is annotated with `#[utoipa::path(...)]` (HTTP method, path and
// responses) and `#[route]`, which self-registers it in the static registry.
// `features.rs`/`router.rs`/`docs.rs` never change when a handler is added.

#[route]
#[utoipa::path(
    get,
    path = "/api/places",
    tag = "places",
    params(
        ("page" = Option<u64>, Query, description = "Page number, 1-based (default 1)", example = 1),
        ("limit" = Option<u64>, Query, description = "Elements per page, 1-100 (default 20)", example = 20),
    ),
    responses(
        (status = 200, description = "Paginated list of places; the total number of elements is returned in the `X-TOTAL-LIST` header", body = [PlaceResponse], headers(("X-TOTAL-LIST" = u64, description = "Total number of places across every page"))),
        (status = 422, description = "Invalid pagination parameters"),
        (status = 503, description = "Database unavailable"),
    ),
)]
pub async fn list_places(
    State(service): State<Arc<PlaceService>>,
    ValidatedQuery(pagination): ValidatedQuery<PaginationQuery>,
) -> AppResult<Response> {
    let page = service.list_page(pagination.pagination()).await?;
    Ok(page.into_response())
}

#[route]
#[utoipa::path(
    post,
    path = "/api/places",
    tag = "places",
    request_body = CreatePlaceDto,
    responses(
        (status = 201, description = "Place created successfully", body = PlaceResponse),
        (status = 400, description = "Invalid request: malformed JSON body"),
        (status = 422, description = "Field validation failed"),
        (status = 503, description = "Database unavailable"),
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
    path = "/api/places/{id}",
    tag = "places",
    params(("id" = String, Path, description = "Place id")),
    responses(
        (status = 200, description = "Place found", body = PlaceResponse),
        (status = 404, description = "Place not found"),
        (status = 503, description = "Database unavailable"),
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
        (status = 400, description = "Invalid request: malformed JSON body"),
        (status = 404, description = "Place not found"),
        (status = 422, description = "Field validation failed"),
        (status = 503, description = "Database unavailable"),
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
        (status = 404, description = "Place not found"),
        (status = 503, description = "Database unavailable"),
    ),
)]
pub async fn delete_place(
    State(service): State<Arc<PlaceService>>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    service.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
