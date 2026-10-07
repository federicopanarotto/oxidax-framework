use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::Router;
use axum_api::api;
use axum_api::api::reports::report_service::ReportService;
use axum_api::db;
use serde_json::{json, Value};
use tower::ServiceExt;

async fn app() -> Router {
    let repository = db::memory_repository().await.expect("memory repository");
    api::router(ReportService::new(repository))
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn reports_full_crud_cycle() {
    let app = app().await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/reports",
            json!({"name": "Vendite mensili", "description": "Report di ottobre"}),
        ))
        .await
        .unwrap();
    let status = response.status();
    let created = json_body(response).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let id = created["id"].as_str().expect("id presente").to_string();
    assert_eq!(created["name"], "Vendite mensili");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/reports/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let fetched = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "{fetched}");
    assert_eq!(fetched["id"], id.as_str());
    assert_eq!(fetched["description"], "Report di ottobre");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/reports")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let list = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    let list = list.as_array().expect("lista di report");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], id.as_str());

    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/reports/{id}"),
            json!({"name": "Vendite trimestrali", "description": "Aggiornato"}),
        ))
        .await
        .unwrap();
    let status = response.status();
    let updated = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["name"], "Vendite trimestrali");
    assert_eq!(updated["id"], id.as_str());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/reports/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/reports/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_without_name_is_rejected() {
    let app = app().await;

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/reports",
            json!({"name": "", "description": "desc"}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert!(body["message"].as_str().unwrap().contains("name"));
}

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
}
