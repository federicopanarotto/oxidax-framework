//! Service business rules (field normalization and 404), without
//! going through the HTTP layer.

use std::sync::Arc;

use axum_api::api::reports::report_dto::CreateReportDto;
use axum_api::api::reports::report_service::ReportService;
use axum_api::errors::AppError;

async fn service() -> Arc<ReportService> {
    crate::support::report_service().await
}

#[tokio::test]
async fn create_trims_name_and_description() {
    let service = service().await;
    let created = service
        .create(CreateReportDto {
            name: "  Vendite mensili  ".into(),
            description: "  Report di prova  ".into(),
            category_id: None,
            place_id: None,
        })
        .await
        .expect("report creato");

    assert_eq!(created.name, "Vendite mensili");
    assert_eq!(created.description, "Report di prova");
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
            category_id: Some("00000000-0000-0000-0000-000000000000".into()),
            place_id: None,
        })
        .await
        .unwrap_err();

    match err {
        AppError::Validation(message) => {
            assert!(
                message.contains("00000000-0000-0000-0000-000000000000"),
                "{message}"
            )
        }
        other => panic!("expected a validation error, got {other:?}"),
    }
}
