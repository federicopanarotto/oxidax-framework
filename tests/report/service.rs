//! Regole di business del service (validazione e 404), senza passare dal
//! livello HTTP.

use axum_api::api::reports::report_dto::CreateReportDto;
use axum_api::api::reports::report_service::ReportService;
use axum_api::errors::AppError;

async fn service() -> ReportService {
    crate::support::report_service().await
}

#[tokio::test]
async fn create_rejects_empty_name() {
    let service = service().await;
    let err = service
        .create(CreateReportDto {
            name: "   ".into(),
            description: "desc".into(),
            category_id: None,
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

#[tokio::test]
async fn create_rejects_unknown_category() {
    let service = service().await;
    let err = service
        .create(CreateReportDto {
            name: "Report".into(),
            description: "desc".into(),
            category_id: Some("missing-category".into()),
        })
        .await
        .unwrap_err();

    match err {
        AppError::Validation(message) => assert!(message.contains("category"), "{message}"),
        other => panic!("atteso errore di validazione, ottenuto {other:?}"),
    }
}
