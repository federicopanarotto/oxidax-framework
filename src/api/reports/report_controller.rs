use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::api::reports::report_dto::{CreateReportDto, ReportResponse, UpdateReportDto};
use crate::api::reports::report_service::ReportService;
use crate::errors::{AppResult, BadRequestResponse, ErrorResponse};
use crate::framework::{ValidatedJson, ValidationErrorResponse, route};

// Route table and OpenAPI spec of the reports, defined point-inline: every
// handler is annotated with `#[utoipa::path(...)]` (HTTP method, path and
// responses) and `#[route]`, which self-registers it in the static registry.
// `features.rs`/`router.rs`/`docs.rs` never change when a handler is added.

#[route]
#[utoipa::path(
    get,
    path = "/api/reports",
    tag = "reports",
    responses(
        (status = 200, description = "List of reports", body = [ReportResponse]),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn list_reports(
    State(service): State<Arc<ReportService>>,
) -> AppResult<Json<Vec<ReportResponse>>> {
    Ok(Json(service.list().await?))
}

#[route]
#[utoipa::path(
    post,
    path = "/api/reports",
    tag = "reports",
    request_body = CreateReportDto,
    responses(
        (status = 201, description = "Report created successfully", body = ReportResponse),
        (status = 400, description = "Invalid request: malformed JSON body or non-existent category", body = BadRequestResponse),
        (status = 422, description = "Field validation failed", body = ValidationErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn create_report(
    State(service): State<Arc<ReportService>>,
    ValidatedJson(dto): ValidatedJson<CreateReportDto>,
) -> AppResult<(StatusCode, Json<ReportResponse>)> {
    let created = service.create(dto).await?;
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
        (status = 404, description = "Report not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
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
        (status = 400, description = "Invalid request: malformed JSON body or non-existent category", body = BadRequestResponse),
        (status = 404, description = "Report not found", body = ErrorResponse),
        (status = 422, description = "Field validation failed", body = ValidationErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
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
        (status = 404, description = "Report not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn delete_report(
    State(service): State<Arc<ReportService>>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    service.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
