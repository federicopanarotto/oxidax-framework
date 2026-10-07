use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};

use crate::api::reports::report_dto::{CreateReportDto, ReportResponse, UpdateReportDto};
use crate::api::reports::report_service::ReportService;
use crate::errors::{AppResult, BadRequestResponse, ErrorResponse};
use crate::lib::{ValidatedJson, ValidationErrorResponse};

/// Tabella delle rotte dei report, costruita sul service iniettato come stato.
///
/// Gli handler sottostanti usano direttamente il [`ReportService`]: non esiste
/// un controller oggetto, sono funzioni `#[utoipa::path]` documentate in
/// OpenAPI. Prima di ogni handler gira l'extractor [`ValidatedJson`], che
/// applica i decoratori `#[validate(...)]` dei DTO e risponde con
/// `{error, details}` se un campo non passa la validazione.
pub fn router(service: Arc<ReportService>) -> Router {
    Router::new()
        .route("/reports", post(create_report).get(list_reports))
        .route(
            "/reports/{id}",
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
