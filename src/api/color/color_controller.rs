use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;

use crate::api::color::color_dto::{CreateColorDto, ColorResponse, UpdateColorDto};
use crate::api::color::color_service::ColorService;
use crate::errors::{AppResult, BadRequestResponse, ErrorResponse};
use crate::framework::{PaginationQuery, ValidatedJson, ValidatedQuery, route};

// Route table and OpenAPI spec of the color feature, defined point-inline:
// every handler is annotated with `#[utoipa::path(...)]` and `#[route]`,
// which self-registers it in the static registry.

#[route]
#[utoipa::path(
    get,
    path = "/api/color",
    tag = "color",
    params(
        ("page" = Option<u64>, Query, description = "Page number, 1-based (default 1)", example = 1),
        ("limit" = Option<u64>, Query, description = "Elements per page, 1-100 (default 20)", example = 20),
    ),
    responses(
        (status = 200, description = "Paginated list of colors; the total is returned in the `X-TOTAL-LIST` header", body = [ColorResponse], headers(("X-TOTAL-LIST" = u64, description = "Total number of colors across every page"))),
        (status = 422, description = "Invalid pagination parameters", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn list_colors(
    State(service): State<Arc<ColorService>>,
    ValidatedQuery(pagination): ValidatedQuery<PaginationQuery>,
) -> AppResult<Response> {
    let page = service.list_page(pagination.pagination()).await?;
    Ok(page.into_response())
}

#[route]
#[utoipa::path(
    post,
    path = "/api/color",
    tag = "color",
    request_body = CreateColorDto,
    responses(
        (status = 201, description = "Color created successfully", body = ColorResponse),
        (status = 400, description = "Invalid request: malformed JSON body", body = BadRequestResponse),
        (status = 422, description = "Field validation failed", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn create_color(
    State(service): State<Arc<ColorService>>,
    ValidatedJson(dto): ValidatedJson<CreateColorDto>,
) -> AppResult<(StatusCode, Json<ColorResponse>)> {
    let created = service.create(dto).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[route]
#[utoipa::path(
    get,
    path = "/api/color/{id}",
    tag = "color",
    params(("id" = String, Path, description = "Color id")),
    responses(
        (status = 200, description = "Color found", body = ColorResponse),
        (status = 404, description = "Color not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn get_color(
    State(service): State<Arc<ColorService>>,
    Path(id): Path<String>,
) -> AppResult<Json<ColorResponse>> {
    Ok(Json(service.get(&id).await?))
}

#[route]
#[utoipa::path(
    put,
    path = "/api/color/{id}",
    tag = "color",
    params(("id" = String, Path, description = "Color id")),
    request_body = UpdateColorDto,
    responses(
        (status = 200, description = "Color updated", body = ColorResponse),
        (status = 400, description = "Invalid request: malformed JSON body", body = BadRequestResponse),
        (status = 404, description = "Color not found", body = ErrorResponse),
        (status = 422, description = "Field validation failed", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn update_color(
    State(service): State<Arc<ColorService>>,
    Path(id): Path<String>,
    ValidatedJson(dto): ValidatedJson<UpdateColorDto>,
) -> AppResult<Json<ColorResponse>> {
    Ok(Json(service.update(&id, dto).await?))
}

#[route]
#[utoipa::path(
    delete,
    path = "/api/color/{id}",
    tag = "color",
    params(("id" = String, Path, description = "Color id")),
    responses(
        (status = 204, description = "Color deleted"),
        (status = 404, description = "Color not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn delete_color(
    State(service): State<Arc<ColorService>>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    service.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
