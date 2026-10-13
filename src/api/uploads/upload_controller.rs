use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;

use crate::api::uploads::upload_dto::{AssetResponse, UploadForm};
use crate::api::uploads::upload_service::UploadService;
use crate::errors::{AppResult, BadRequestResponse, ErrorResponse};
use crate::framework::{CurrentUser, PaginationQuery, Upload, ValidatedQuery, route};

// Route table and OpenAPI spec of the uploads, defined point-inline: every
// handler is annotated with `#[utoipa::path(...)]` and `#[route]`, which
// self-registers it in the static registry.

#[route]
#[utoipa::path(
    get,
    path = "/api/upload",
    tag = "uploads",
    params(
        ("page" = Option<u64>, Query, description = "Page number, 1-based (default 1)", example = 1),
        ("limit" = Option<u64>, Query, description = "Elements per page, 1-100 (default 20)", example = 20),
    ),
    responses(
        (status = 200, description = "Paginated list of assets; the total number of elements is returned in the `X-TOTAL-LIST` header", body = [AssetResponse], headers(("X-TOTAL-LIST" = u64, description = "Total number of assets across every page"))),
        (status = 422, description = "Invalid pagination parameters", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn list_uploaded_files(
    State(service): State<Arc<UploadService>>,
    ValidatedQuery(pagination): ValidatedQuery<PaginationQuery>,
) -> AppResult<Response> {
    let page = service.list_page(pagination.pagination()).await?;
    Ok(page.into_response())
}

#[route]
#[utoipa::path(
    post,
    path = "/api/upload",
    tag = "uploads",
    request_body(content = UploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "File uploaded successfully", body = AssetResponse),
        (status = 400, description = "Invalid request: missing `file` field or malformed multipart body", body = BadRequestResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 413, description = "File too large or too many files", body = ErrorResponse),
        (status = 415, description = "File extension not allowed", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
    security(
        ("accessToken" = []),
    ),
)]
#[IsGranted]
pub async fn upload_file(
    State(service): State<Arc<UploadService>>,
    current: CurrentUser,
    form: Upload,
) -> AppResult<(StatusCode, Json<AssetResponse>)> {
    let asset = service.create(form, Some(current.user_id)).await?;
    Ok((StatusCode::CREATED, Json(asset)))
}

#[route]
#[utoipa::path(
    get,
    path = "/api/upload/{id}",
    tag = "uploads",
    params(("id" = String, Path, description = "Asset id")),
    responses(
        (status = 200, description = "The stored file, streamed with its original content type"),
        (status = 404, description = "Asset not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn download_file(
    State(service): State<Arc<UploadService>>,
    Path(id): Path<String>,
) -> AppResult<Response> {
    let (asset, path) = service.open(&id).await?;
    crate::lib::upload::stream_file(path, &asset.content_type, &asset.original_name).await
}

#[route]
#[utoipa::path(
    delete,
    path = "/api/upload/{id}",
    tag = "uploads",
    params(("id" = String, Path, description = "Asset id")),
    responses(
        (status = 204, description = "Asset deleted"),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "The asset belongs to another user", body = ErrorResponse),
        (status = 404, description = "Asset not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
    security(
        ("accessToken" = []),
    ),
)]
#[IsGranted]
pub async fn delete_file(
    State(service): State<Arc<UploadService>>,
    Path(id): Path<String>,
    current: CurrentUser,
) -> AppResult<StatusCode> {
    service.delete(&id, &current).await?;
    Ok(StatusCode::NO_CONTENT)
}
