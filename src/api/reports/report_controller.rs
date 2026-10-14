use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;

use crate::api::reports::report_dto::{CreateReportDto, ReportResponse, UpdateReportDto};
use crate::api::reports::report_service::ReportService;
use crate::errors::AppResult;
use crate::framework::{CurrentUser, PaginationQuery, ValidatedJson, ValidatedQuery, route};

// Route table and OpenAPI spec of the reports, defined point-inline: every
// handler is annotated with `#[utoipa::path(...)]` (HTTP method, path and
// responses) and `#[route]`, which self-registers it in the static registry.
// `features.rs`/`router.rs`/`docs.rs` never change when a handler is added.

#[route]
#[utoipa::path(
    get,
    path = "/api/reports",
    tag = "reports",
    params(
        ("page" = Option<u64>, Query, description = "Page number, 1-based (default 1)", example = 1),
        ("limit" = Option<u64>, Query, description = "Elements per page, 1-100 (default 20)", example = 20),
    ),
    responses(
        (status = 200, description = "Paginated list of reports; the total number of elements is returned in the `X-TOTAL-LIST` header", body = [ReportResponse], headers(("X-TOTAL-LIST" = u64, description = "Total number of reports across every page"))),
        (status = 422, description = "Invalid pagination parameters"),
        (status = 503, description = "Database unavailable"),
    ),
)]
pub async fn list_reports(
    State(service): State<Arc<ReportService>>,
    ValidatedQuery(pagination): ValidatedQuery<PaginationQuery>,
) -> AppResult<Response> {
    let page = service.list_page(pagination.pagination()).await?;
    Ok(page.into_response())
}

#[route]
#[utoipa::path(
    post,
    path = "/api/reports",
    tag = "reports",
    request_body = CreateReportDto,
    responses(
        (status = 201, description = "Report created successfully", body = ReportResponse),
        (status = 400, description = "Invalid request: malformed JSON body or non-existent category"),
        (status = 401, description = "Missing or invalid access token"),
        (status = 422, description = "Field validation failed"),
        (status = 503, description = "Database unavailable"),
    ),
    security(
        ("accessToken" = []),
    ),
)]
#[IsGranted]
pub async fn create_report(
    State(service): State<Arc<ReportService>>,
    current: CurrentUser,
    ValidatedJson(dto): ValidatedJson<CreateReportDto>,
) -> AppResult<(StatusCode, Json<ReportResponse>)> {
    let created = service.create(&current.user_id, dto).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[route]
#[utoipa::path(
    get,
    path = "/api/reports/{id}",
    tag = "reports",
    params(("id" = String, Path, description = "Report id")),
    responses(
        (status = 200, description = "Report found", body = ReportResponse),
        (status = 404, description = "Report not found"),
        (status = 503, description = "Database unavailable"),
    ),
)]
pub async fn get_report(
    State(service): State<Arc<ReportService>>,
    Path(id): Path<String>,
) -> AppResult<Json<ReportResponse>> {
    Ok(Json(service.get(&id).await?))
}

#[route]
#[utoipa::path(
    put,
    path = "/api/reports/{id}",
    tag = "reports",
    params(("id" = String, Path, description = "Report id")),
    request_body = UpdateReportDto,
    responses(
        (status = 200, description = "Report updated", body = ReportResponse),
        (status = 400, description = "Invalid request: malformed JSON body or non-existent category"),
        (status = 401, description = "Missing or invalid access token"),
        (status = 404, description = "Report not found"),
        (status = 422, description = "Field validation failed"),
        (status = 503, description = "Database unavailable"),
    ),
    security(
        ("accessToken" = []),
    ),
)]
#[IsGranted]
pub async fn update_report(
    State(service): State<Arc<ReportService>>,
    Path(id): Path<String>,
    ValidatedJson(dto): ValidatedJson<UpdateReportDto>,
) -> AppResult<Json<ReportResponse>> {
    Ok(Json(service.update(&id, dto).await?))
}

#[route]
#[utoipa::path(
    delete,
    path = "/api/reports/{id}",
    tag = "reports",
    params(("id" = String, Path, description = "Report id")),
    responses(
        (status = 204, description = "Report deleted"),
        (status = 401, description = "Missing or invalid access token"),
        (status = 404, description = "Report not found"),
        (status = 503, description = "Database unavailable"),
    ),
    security(
        ("accessToken" = []),
    ),
)]
#[IsGranted]
pub async fn delete_report(
    State(service): State<Arc<ReportService>>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    service.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
