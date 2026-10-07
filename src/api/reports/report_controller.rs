use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::api::reports::report_dto::{CreateReportDto, ReportResponse, UpdateReportDto};
use crate::api::reports::report_service::ReportService;
use crate::errors::{AppJson, AppResult, ErrorResponse};

/// Controller dei report: incapsula il service ed espone gli endpoint.
///
/// I metodi contengono la logica HTTP (status code + delega al service);
/// le funzioni con `#[utoipa::path]` sono i handler axum, sottili wrapper
/// che estraggono la richiesta e documentano l'endpoint in OpenAPI.
#[derive(Clone)]
pub struct ReportController {
    service: ReportService,
}

impl ReportController {
    pub fn new(service: ReportService) -> Self {
        Self { service }
    }

    /// Crea un report: 201 con il report appena inserito.
    pub async fn create(&self, dto: CreateReportDto) -> AppResult<(StatusCode, ReportResponse)> {
        let created = self.service.create(dto).await?;
        Ok((StatusCode::CREATED, created))
    }

    /// Elenca tutti i report.
    pub async fn list(&self) -> AppResult<Vec<ReportResponse>> {
        self.service.list().await
    }

    /// Recupera un report per id.
    pub async fn get(&self, id: &str) -> AppResult<ReportResponse> {
        self.service.get(id).await
    }

    /// Sostituisce un report esistente.
    pub async fn update(&self, id: &str, dto: UpdateReportDto) -> AppResult<ReportResponse> {
        self.service.update(id, dto).await
    }

    /// Elimina un report: 204 senza corpo.
    pub async fn delete(&self, id: &str) -> AppResult<StatusCode> {
        self.service.delete(id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
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
pub async fn create_report(
    State(controller): State<ReportController>,
    AppJson(dto): AppJson<CreateReportDto>,
) -> AppResult<(StatusCode, Json<ReportResponse>)> {
    let (status, created) = controller.create(dto).await?;
    Ok((status, Json(created)))
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
pub async fn list_reports(
    State(controller): State<ReportController>,
) -> AppResult<Json<Vec<ReportResponse>>> {
    Ok(Json(controller.list().await?))
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
pub async fn get_report(
    State(controller): State<ReportController>,
    Path(id): Path<String>,
) -> AppResult<Json<ReportResponse>> {
    Ok(Json(controller.get(&id).await?))
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
pub async fn update_report(
    State(controller): State<ReportController>,
    Path(id): Path<String>,
    AppJson(dto): AppJson<UpdateReportDto>,
) -> AppResult<Json<ReportResponse>> {
    Ok(Json(controller.update(&id, dto).await?))
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
pub async fn delete_report(
    State(controller): State<ReportController>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    controller.delete(&id).await
}
