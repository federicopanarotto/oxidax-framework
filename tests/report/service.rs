//! Regole di business del service (validazione e 404), senza passare dal
//! livello HTTP.

use axum_api::api::reports::report_dto::CreateReportDto;
use axum_api::api::reports::report_service::ReportService;
use axum_api::db;
use axum_api::errors::AppError;

async fn service() -> ReportService {
    let repository = db::memory_repository().await.expect("memory repository");
    ReportService::new(repository)
}

#[tokio::test]
async fn create_rejects_empty_name() {
    let service = service().await;
    let err = service
        .create(CreateReportDto {
            name: "   ".into(),
            description: "desc".into(),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Validation(_)), "{err:?}");
}

#[tokio::test]
async fn get_missing_report_is_not_found() {
    let service = service().await;
    let err = service.get("does-not-exist").await.unwrap_err();

    assert!(matches!(err, AppError::NotFound(_)), "{err:?}");
}
