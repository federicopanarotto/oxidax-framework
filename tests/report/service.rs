//! Service business rules (field normalization and 404), without
//! going through the HTTP layer.

use std::sync::Arc;

use axum_api::api::reports::report_dto::CreateReportDto;
use axum_api::api::reports::report_service::ReportService;
use axum_api::errors::AppError;

async fn service() -> Arc<ReportService> {
    crate::support::report_service().await
}

/// Id of the caller passed to `create`: it comes from the token, so at the
/// service level it is just a string (no validation against the database).
const CREATOR: &str = "550e8400-e29b-41d4-a716-446655440000";

#[tokio::test]
async fn create_trims_name_and_description() {
    let service = service().await;
    let created = service
        .create(
            CREATOR,
            CreateReportDto {
                name: "  Vendite mensili  ".into(),
                description: "  Report di prova  ".into(),
                category_id: None,
                place_id: None,
            },
        )
        .await
        .expect("report creato");

    assert_eq!(created.name, "Vendite mensili");
    assert_eq!(created.description, "Report di prova");
}

#[tokio::test]
async fn creator_degrades_to_null_when_the_user_does_not_exist() {
    let service = service().await;
    let created = service
        .create(
            "00000000-0000-0000-0000-000000000000",
            CreateReportDto {
                name: "Report".into(),
                description: "desc".into(),
                category_id: None,
                place_id: None,
            },
        )
        .await
        .expect("report creato");

    // The creator comes from the verified token: it is not re-checked
    // against the database, and the decoration degrades to null if the user
    // no longer exists.
    assert!(created.created_by.is_none());
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
        .create(
            CREATOR,
            CreateReportDto {
                name: "Report".into(),
                description: "desc".into(),
                category_id: Some("00000000-0000-0000-0000-000000000000".into()),
                place_id: None,
            },
        )
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
