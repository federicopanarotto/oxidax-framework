use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::api::reports::report_dto::{
    CreateReportDto, ErrorResponse, ReportResponse, UpdateReportDto,
};
use crate::api::reports::report_service::ReportService;

pub fn router(service: ReportService) -> Router {
    Router::new()
        .route("/", post(create_report).get(list_reports))
        .route(
            "/{id}",
            get(get_report).put(update_report).delete(delete_report),
        )
        .with_state(service)
}

#[utoipa::path(
    post,
    path = "/api/reports",
    tag = "reports",
    request_body = CreateReportDto,
    responses(
        (status = 201, description = "Report created", body = ReportResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
async fn create_report(
    State(service): State<ReportService>,
    Json(dto): Json<CreateReportDto>,
) -> Result<(StatusCode, Json<ReportResponse>), crate::error::AppError> {
    let created = service.create(dto).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    get,
    path = "/api/reports",
    tag = "reports",
    responses(
        (status = 200, description = "List of reports", body = [ReportResponse]),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
async fn list_reports(
    State(service): State<ReportService>,
) -> Result<Json<Vec<ReportResponse>>, crate::error::AppError> {
    Ok(Json(service.list().await?))
}

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
async fn get_report(
    State(service): State<ReportService>,
    Path(id): Path<String>,
) -> Result<Json<ReportResponse>, crate::error::AppError> {
    Ok(Json(service.get(&id).await?))
}

#[utoipa::path(
    put,
    path = "/api/reports/{id}",
    tag = "reports",
    params(("id" = String, Path, description = "Report id")),
    request_body = UpdateReportDto,
    responses(
        (status = 200, description = "Report updated", body = ReportResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 404, description = "Report not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
async fn update_report(
    State(service): State<ReportService>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateReportDto>,
) -> Result<Json<ReportResponse>, crate::error::AppError> {
    Ok(Json(service.update(&id, dto).await?))
}

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
async fn delete_report(
    State(service): State<ReportService>,
    Path(id): Path<String>,
) -> Result<StatusCode, crate::error::AppError> {
    service.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
