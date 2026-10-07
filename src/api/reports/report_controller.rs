use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use utoipa::openapi::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::api::reports::report_dto::{CreateReportDto, ReportResponse, UpdateReportDto};
use crate::api::reports::report_service::ReportService;
use crate::errors::{AppResult, BadRequestResponse, ErrorResponse};
use crate::lib::{ValidatedJson, ValidationErrorResponse};

/// Route table (axum) and OpenAPI spec for reports, defined right here,
/// without an object controller: the handlers are `#[utoipa::path]`
/// functions that use [`ReportService`] as state.
///
/// Registration in the DI container and in the feature registry (`order` 1)
/// live in `mod.rs`, so `features.rs`/`router.rs`/`docs.rs` never change.
pub fn build_router(service: Arc<ReportService>) -> (Router, OpenApi) {
    OpenApiRouter::new()
        .routes(routes!(create_report, list_reports))
        .routes(routes!(get_report, update_report, delete_report))
        .with_state(service)
        .split_for_parts()
}

#[utoipa::path(
    post,
    path = "/api/reports",
    tag = "reports",
    request_body = CreateReportDto,
    responses(
        (status = 201, description = "Report creato con successo", body = ReportResponse),
        (status = 400, description = "Richiesta non valida: body JSON malformato o categoria inesistente", body = BadRequestResponse),
        (status = 422, description = "Validazione dei campi non superata", body = ValidationErrorResponse),
        (status = 503, description = "Database non disponibile", body = ErrorResponse),
    ),
)]
pub async fn create_report(
    State(service): State<Arc<ReportService>>,
    ValidatedJson(dto): ValidatedJson<CreateReportDto>,
) -> AppResult<(StatusCode, Json<ReportResponse>)> {
    let created = service.create(dto).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    get,
    path = "/api/reports",
    tag = "reports",
    responses(
        (status = 200, description = "Elenco dei report", body = [ReportResponse]),
        (status = 503, description = "Database non disponibile", body = ErrorResponse),
    ),
)]
pub async fn list_reports(
    State(service): State<Arc<ReportService>>,
) -> AppResult<Json<Vec<ReportResponse>>> {
    Ok(Json(service.list().await?))
}

#[utoipa::path(
    get,
    path = "/api/reports/{id}",
    tag = "reports",
    params(("id" = String, Path, description = "Id del report")),
    responses(
        (status = 200, description = "Report trovato", body = ReportResponse),
        (status = 404, description = "Report non trovato", body = ErrorResponse),
        (status = 503, description = "Database non disponibile", body = ErrorResponse),
    ),
)]
pub async fn get_report(
    State(service): State<Arc<ReportService>>,
    Path(id): Path<String>,
) -> AppResult<Json<ReportResponse>> {
    Ok(Json(service.get(&id).await?))
}

#[utoipa::path(
    put,
    path = "/api/reports/{id}",
    tag = "reports",
    params(("id" = String, Path, description = "Id del report")),
    request_body = UpdateReportDto,
    responses(
        (status = 200, description = "Report aggiornato", body = ReportResponse),
        (status = 400, description = "Richiesta non valida: body JSON malformato o categoria inesistente", body = BadRequestResponse),
        (status = 404, description = "Report non trovato", body = ErrorResponse),
        (status = 422, description = "Validazione dei campi non superata", body = ValidationErrorResponse),
        (status = 503, description = "Database non disponibile", body = ErrorResponse),
    ),
)]
pub async fn update_report(
    State(service): State<Arc<ReportService>>,
    Path(id): Path<String>,
    ValidatedJson(dto): ValidatedJson<UpdateReportDto>,
) -> AppResult<Json<ReportResponse>> {
    Ok(Json(service.update(&id, dto).await?))
}

#[utoipa::path(
    delete,
    path = "/api/reports/{id}",
    tag = "reports",
    params(("id" = String, Path, description = "Id del report")),
    responses(
        (status = 204, description = "Report eliminato"),
        (status = 404, description = "Report non trovato", body = ErrorResponse),
        (status = 503, description = "Database non disponibile", body = ErrorResponse),
    ),
)]
pub async fn delete_report(
    State(service): State<Arc<ReportService>>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    service.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
