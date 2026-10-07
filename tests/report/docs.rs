//! Health, Swagger UI e specifica OpenAPI.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use crate::support::{app, json_body};

#[tokio::test]
async fn swagger_and_health_are_available() {
    let app = app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/swagger-ui/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api-docs/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let spec = json_body(response).await;
    let paths = spec["paths"].as_object().unwrap();
    assert!(paths.contains_key("/api/reports"));
    assert!(paths.contains_key("/api/reports/{id}"));
    assert!(paths.contains_key("/api/categories"));
    assert!(paths.contains_key("/api/categories/{id}"));

    let schemas = spec["components"]["schemas"].as_object().unwrap();
    assert!(schemas.contains_key("ErrorResponse"));
    assert!(schemas.contains_key("ReportResponse"));
    assert!(schemas.contains_key("CategoryResponse"));
    assert!(schemas.contains_key("ValidationErrorResponse"));

    let create_report = &spec["paths"]["/api/reports"]["post"];
    assert!(
        create_report["responses"]["422"].is_object(),
        "risposta 422 documentata per la creazione dei report"
    );
    assert_eq!(
        create_report["responses"]["422"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/ValidationErrorResponse"
    );
}

#[tokio::test]
async fn openapi_spec_is_exported_on_startup() {
    let _app = app().await;

    let file = std::fs::read_to_string("docs/openapi.json").expect("openapi.json esportato");
    let spec: serde_json::Value = serde_json::from_str(&file).unwrap();
    assert!(spec["openapi"].is_string());
    assert!(
        spec["paths"]
            .as_object()
            .unwrap()
            .contains_key("/api/reports")
    );
    assert!(
        spec["paths"]
            .as_object()
            .unwrap()
            .contains_key("/api/categories")
    );
}
