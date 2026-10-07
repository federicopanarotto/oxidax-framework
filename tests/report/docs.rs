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
                .uri("/health")
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
}
