//! Ciclo completo CRUD via HTTP e gestione degli errori di validazione.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use crate::support::{app, json_body, json_request};

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
    assert_eq!(body["code"], "validation_error");
    assert!(body["message"].as_str().unwrap().contains("name"));
}

#[tokio::test]
async fn malformed_json_uses_error_pipeline() {
    let app = app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/reports")
                .header("content-type", "application/json")
                .body(Body::from("{non-json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["code"], "validation_error");
    assert!(body["message"].is_string());
}

#[tokio::test]
async fn unknown_route_returns_uniform_error() {
    let app = app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/inesistente")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(body["code"], "not_found");
}
